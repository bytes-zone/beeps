use crate::auth::Claims;
use crate::conn::Conn;
use crate::error::Error;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::{query, Acquire};

#[derive(Debug, Deserialize)]
pub struct Req {
    name: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resp {
    id: i32,
}

#[tracing::instrument]
pub async fn handler(
    claims: Claims,
    Conn(mut conn): Conn,
    req: Json<Req>,
) -> Result<Json<Resp>, Error> {
    let mut tx = conn.begin().await?;

    // 1. look up the devices already in the document
    let devices = query!(
        "SELECT name FROM devices WHERE document_id = $1",
        claims.document_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tracing::debug!(?devices, "devices in account");

    // 2. error if one has the same name
    for device in devices {
        if device.name == req.name {
            return Err(Error::bad_request(&format!(
                "a device named {} already exists",
                req.name
            )));
        }
    }

    // 3. create a device with the name with a node id of one more than existing devices
    let new_row = query!(
        "INSERT INTO devices (document_id, name) VALUES ($1, $2) RETURNING id",
        claims.document_id,
        req.name,
    )
    .fetch_one(&mut *tx)
    .await?;

    tracing::debug!(?new_row, "new row created");

    // all done! Commit and return.
    tx.commit().await?;

    Ok(Json(Resp { id: new_row.id }))
}

#[cfg(test)]
mod test {
    use super::super::test::Doc;
    use super::*;
    use crate::conn::Conn;
    use sqlx::{pool::PoolConnection, Pool, Postgres};

    #[test_log::test(sqlx::test)]
    async fn enrolls_in_empty_document(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        let req = Json(Req {
            name: "test".to_string(),
        });

        let res = handler(doc.claims(), Conn(conn), req).await.unwrap();

        assert_eq!(res.0, Resp { id: 1 });
    }

    #[test_log::test(sqlx::test)]
    fn enrolls_with_same_name_fails(mut conn: PoolConnection<Postgres>) {
        let doc = Doc::create(&mut conn).await;

        query!(
            "INSERT INTO devices (document_id, name) VALUES ($1, $2)",
            doc.document_id,
            "test"
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let req = Json(Req {
            name: "test".to_string(),
        });

        let res = handler(doc.claims(), Conn(conn), req).await.unwrap_err();

        assert_eq!(res.status_code, 400);
        assert_eq!(res.message, "a device named test already exists")
    }

    #[test_log::test(sqlx::test)]
    fn returns_unique_device_ids(pool: Pool<Postgres>) {
        let doc = Doc::create(&mut pool.acquire().await.unwrap()).await;

        let req = Json(Req {
            name: "test".to_string(),
        });

        let res = handler(doc.claims(), Conn(pool.acquire().await.unwrap()), req)
            .await
            .unwrap();

        assert_eq!(res.0, Resp { id: 1 });

        let req = Json(Req {
            name: "test2".to_string(),
        });

        let res = handler(doc.claims(), Conn(pool.acquire().await.unwrap()), req)
            .await
            .unwrap();

        assert_eq!(res.0, Resp { id: 2 });
    }
}
