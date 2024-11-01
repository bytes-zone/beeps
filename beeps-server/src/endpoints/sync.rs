use super::presync::LatestEvents;
use crate::auth::Claims;
use crate::conn::Conn;
use crate::error::Error;
use axum::Json;
use chrono::{DateTime, Utc};
use common::hlc::Hlc;
use common::log::TimestampedOp;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, QueryBuilder, Row};

#[derive(Debug, Deserialize)]
pub struct Req {
    starting: LatestEvents,
    timestamped_ops: Vec<TimestampedOp>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resp {
    timestamped_ops: Vec<TimestampedOp>,
}

#[tracing::instrument(skip(req))]
pub async fn handler(
    claims: Claims,
    Conn(mut conn): Conn,
    req: Json<Req>,
) -> Result<Json<Resp>, Error> {
    let mut tx = conn.begin().await.map_err(|err| {
        tracing::error!(?err, "failed to acquire connection");
        Error::internal_server_error("failed to acquire connection")
    })?;

    // 1. get newer items that the client doesn't have
    let mut new_rows = QueryBuilder::new(
        "SELECT timestamp, counter, device_id, op FROM operations WHERE document_id = ",
    );
    new_rows.push_bind(claims.document_id);
    if !req.starting.is_empty() {
        new_rows.push(" AND (");

        // we can bind a max of 2^16 params. We already used one, and each clock
        // has four binds, giving us this limit.
        if req.starting.len() > 65535 / 4 - 1 {
            return Err(Error::bad_request("too many clocks"));
        }

        for (i, clock) in req.starting.values().enumerate() {
            if i != 0 {
                new_rows.push(" OR ");
            }
            new_rows.push("(timestamp, counter, device_id) > (");
            new_rows.push_bind(clock.timestamp);
            new_rows.push(", ");
            new_rows.push_bind(clock.counter);
            new_rows.push(", ");
            new_rows.push_bind(clock.node);
            new_rows.push(")");
        }

        new_rows.push(" OR device_id NOT IN (");
        for (i, clock) in req.starting.values().enumerate() {
            if i != 0 {
                new_rows.push(", ");
            }
            new_rows.push_bind(clock.node);
        }

        new_rows.push("))");
    }

    let mut new_timestamped_ops = Vec::with_capacity(32);
    for row in new_rows.build().fetch_all(&mut *tx).await.map_err(|err| {
        tracing::error!(?err, "failed to query latest ops");
        Error::internal_server_error("failed to query latest ops")
    })? {
        new_timestamped_ops.push(TimestampedOp {
            timestamp: Hlc {
                timestamp: row.try_get::<DateTime<Utc>, &str>("timestamp")?,
                counter: row.try_get("counter")?,
                node: row.try_get("device_id")?,
            },
            op: serde_json::from_value(row.try_get("op")?).map_err(|err| {
                tracing::error!(?err, "failed to deserialize op from database");
                Error::internal_server_error("bad format for op in database")
            })?,
        })
    }

    // insert the items we were sent
    if !req.timestamped_ops.is_empty() {
        // we can bind a max of 2^16 params. Each row has four binds, giving us this limit.
        if req.timestamped_ops.len() > 65535 / 4 {
            return Err(Error::bad_request("too many events"));
        }

        let mut insert_query = QueryBuilder::new(
            "INSERT INTO operations (document_id, timestamp, counter, device_id, op)",
        );
        insert_query.push_values(req.0.timestamped_ops, |mut b, ts_op| {
            b.push(claims.document_id);
            b.push_bind(ts_op.timestamp.timestamp);
            b.push_bind(ts_op.timestamp.counter);
            b.push_bind(ts_op.timestamp.node);
            b.push_bind(
                serde_json::to_value(ts_op.op)
                    .expect("op to be a valid op, since it just deserialized"),
            );
        });

        insert_query
            .build()
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                tracing::error!(?err, "failed to insert ops");
                Error::internal_server_error("failed to insert ops")
            })?;
    }

    // 1. look up the latest messages in the database again and make sure they match the ones we were sent in the request

    // 2. insert new messages with a safety check that they are newer than the ones we already have
    // 3. send back messages newer than the ones in req.starting

    // all done! Commit and return.
    tx.commit().await.map_err(|err| {
        tracing::error!(?err, "could not commit transaction");
        Error::internal_server_error("error querying")
    })?;

    Ok(Json(Resp {
        timestamped_ops: new_timestamped_ops,
    }))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::conn::Conn;
    use crate::endpoints::test::Doc;
    use common::{hlc::Hlc, op::Op};
    use sqlx::{pool::PoolConnection, query, Pool, Postgres};

    #[test_log::test(sqlx::test)]
    async fn empty_doc(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let req = Req {
            starting: Default::default(),
            timestamped_ops: vec![],
        };

        let res = handler(doc.claims(), Conn(conn), Json(req)).await.unwrap();
        assert_eq!(
            res.0,
            Resp {
                timestamped_ops: vec![]
            }
        );
    }

    #[test_log::test(sqlx::test)]
    async fn op_client_doesnt_have_empty(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let device_id = doc.add_device(&mut conn, "test").await;
        let timestamp = Hlc::new(device_id);

        let op = Op::AddPing {
            when: chrono::Utc::now(),
        };

        doc.add_op(&mut conn, &timestamp, &op).await;

        let req = Req {
            starting: Default::default(),
            timestamped_ops: vec![],
        };

        let res = handler(doc.claims(), Conn(conn), Json(req)).await.unwrap();
        assert_eq!(
            res.0,
            Resp {
                timestamped_ops: vec![TimestampedOp { timestamp, op }]
            }
        )
    }

    #[test_log::test(sqlx::test)]
    async fn op_client_doesnt_have_partial(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let device_id = doc.add_device(&mut conn, "test").await;

        let when = chrono::Utc::now() - chrono::Duration::minutes(1);

        // event the client will already have seen
        let seen = TimestampedOp {
            timestamp: Hlc::new_at(device_id, when - chrono::Duration::seconds(1)),
            op: Op::SetTag {
                when: when - chrono::Duration::seconds(1),
                tag: "NO".to_string(),
            },
        };
        doc.add_op(&mut conn, &seen.timestamp, &seen.op).await;

        // event the client will not have seen yet
        let unseen = TimestampedOp {
            timestamp: Hlc::new_at(device_id, when + chrono::Duration::seconds(1)),
            op: Op::SetTag {
                when: when + chrono::Duration::seconds(1),
                tag: "YES".to_string(),
            },
        };
        doc.add_op(&mut conn, &unseen.timestamp, &unseen.op).await;

        let req = Req {
            starting: HashMap::from([(device_id, Hlc::new_at(device_id, when))]),
            timestamped_ops: vec![],
        };

        let res = handler(doc.claims(), Conn(conn), Json(req)).await.unwrap();
        assert_eq!(
            res.0,
            Resp {
                timestamped_ops: vec![unseen]
            }
        )
    }

    #[test_log::test(sqlx::test)]
    async fn op_server_doesnt_have(pool: Pool<Postgres>) {
        let doc = Doc::create(&mut pool.acquire().await.unwrap()).await;

        let device_id = doc
            .add_device(&mut pool.acquire().await.unwrap(), "test")
            .await;
        let timestamp = Hlc::new(device_id);

        let op = Op::AddPing {
            when: chrono::Utc::now(),
        };

        let timestamped_op = TimestampedOp {
            timestamp: timestamp.clone(),
            op: op.clone(),
        };

        let req = Req {
            starting: Default::default(),
            timestamped_ops: vec![timestamped_op],
        };

        let res = handler(doc.claims(), Conn(pool.acquire().await.unwrap()), Json(req))
            .await
            .unwrap();

        assert_eq!(
            res.0,
            Resp {
                timestamped_ops: vec![]
            }
        );

        let new_row = query!("SELECT timestamp, counter, device_id, op FROM operations")
            .fetch_one(&mut *pool.acquire().await.unwrap())
            .await
            .unwrap();

        assert_eq!(new_row.timestamp, timestamp.timestamp);
        assert_eq!(new_row.counter, timestamp.counter);
        assert_eq!(new_row.device_id, timestamp.node);
        assert_eq!(new_row.op, serde_json::to_value(op).unwrap());
    }
}
