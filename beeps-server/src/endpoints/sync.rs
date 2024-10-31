use super::presync::LatestEvents;
use crate::auth::Claims;
use crate::conn::Conn;
use crate::error::Error;
use axum::Json;
use common::log::TimestampedOp;
use serde::{Deserialize, Serialize};

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
    Conn(mut _conn): Conn,
    req: Json<Req>,
) -> Result<Json<Resp>, Error> {
    tracing::debug!(?req, "req");

    // 1. look up the latest messages in the database again and make sure they match the ones we were sent in the request
    // 2. insert new messages with a safety check that they are newer than the ones we already have
    // 3. send back messages newer than the ones in req.starting

    Ok(Json(Resp {
        timestamped_ops: vec![],
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conn::Conn;
    use crate::endpoints::test::Doc;
    use sqlx::{pool::PoolConnection, Postgres};

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
}
