use std::collections::HashMap;

use crate::auth::Claims;
use crate::conn::Conn;
use crate::error::Error;
use axum::Json;
use common::hlc::Hlc;
use sqlx::query;

pub type LatestEvents = HashMap<i64, Hlc>;

#[tracing::instrument]
pub async fn handler(claims: Claims, Conn(mut conn): Conn) -> Result<Json<LatestEvents>, Error> {
    let rows = query!(
        r#"
        SELECT DISTINCT ON (device_id)
          device_id,
          "timestamp",
          counter
        FROM operations
        WHERE document_id = $1
        ORDER BY
            device_id,
            "timestamp" DESC,
            counter DESC
        "#,
        claims.document_id
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut out = HashMap::with_capacity(rows.len());
    for row in rows {
        out.insert(
            row.device_id,
            Hlc {
                timestamp: row.timestamp,
                counter: row.counter,
                node: row.device_id,
            },
        );
    }

    Ok(Json(out))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::endpoints::test::Doc;
    use crate::{conn::Conn, endpoints::test::trunc_ms};
    use serde_json::json;
    use sqlx::{pool::PoolConnection, Postgres};

    #[test_log::test(sqlx::test)]
    async fn empty_doc(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let res = handler(doc.claims(), Conn(conn)).await.unwrap();

        assert_eq!(res.0.len(), 0);
    }

    #[test_log::test(sqlx::test)]
    async fn get_latest_for_each_device(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let device_id = doc.add_device(&mut conn, "test").await;

        let now = chrono::Utc::now();

        query!(
            "INSERT INTO operations (document_id, timestamp, counter, device_id, op) VALUES ($1, $2, $3, $4, $5)",
            doc.document_id,
            now,
            0,
            device_id,
            json!("0")
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        query!(
            "INSERT INTO operations (document_id, timestamp, counter, device_id, op) VALUES ($1, $2, $3, $4, $5)",
            doc.document_id,
            now,
            1,
            device_id,
            json!("0")
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let device_id_2 = doc.add_device(&mut conn, "test2").await;

        query!(
            "INSERT INTO operations (document_id, timestamp, counter, device_id, op) VALUES ($1, $2, $3, $4, $5)",
            doc.document_id,
            now,
            0,
            device_id_2,
            json!("0")
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let res = handler(doc.claims(), Conn(conn)).await.unwrap();

        assert_eq!(
            res.0,
            HashMap::from([
                (
                    device_id,
                    Hlc {
                        timestamp: trunc_ms(now),
                        counter: 1,
                        node: device_id
                    }
                ),
                (
                    device_id_2,
                    Hlc {
                        timestamp: trunc_ms(now),
                        counter: 0,
                        node: device_id_2
                    }
                )
            ])
        )
    }
}
