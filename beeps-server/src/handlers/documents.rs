use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::sync::documents;
use sqlx::query_as;

#[tracing::instrument]
pub async fn handler(Conn(mut conn): Conn, claims: Claims) -> Result<Json<documents::Resp>, Error> {
    let documents = query_as!(
        documents::Document,
        "SELECT documents.id as id, updated_at, created_at \
        FROM documents \
        JOIN accounts ON accounts.id = documents.owner_id \
        WHERE accounts.email = $1",
        claims.sub,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(Json(documents::Resp { documents }))
}

#[cfg(test)]
mod test {
    use sqlx::{pool::PoolConnection, Postgres};

    use crate::handlers::test::TestDoc;

    use super::*;

    #[test_log::test(sqlx::test)]
    fn test_success(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let resp = handler(Conn(conn), doc.claims()).await.unwrap();

        assert_eq!(
            resp.0.documents.first().map(|d| d.id),
            Some(doc.document_id)
        );
    }
}
