use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::http::StatusCode;
use axum::Json;
use beeps_core::sync::push;

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Json(req): Json<push::Req>,
) -> Result<Json<push::Resp>, Error> {
    // Validate that the user owns the document
    let authed_document = sqlx::query!(
        "SELECT documents.id FROM documents \
        JOIN accounts ON accounts.id = documents.owner_id \
        WHERE accounts.email = $1 AND documents.id = $2",
        claims.sub,
        req.document_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

    bail_if!(
        authed_document.is_none(),
        "Document not found",
        StatusCode::NOT_FOUND
    );

    Ok(Json(push::Resp {}))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handlers::test::TestDoc;
    use beeps_core::Document;
    use sqlx::{pool::PoolConnection, Postgres};

    #[test_log::test(sqlx::test)]
    fn test_unknown_document_not_authorized(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let err = handler(
            Conn(conn),
            doc.claims(),
            Json(push::Req {
                document_id: doc.document_id + 1,
                document: Document::default(),
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(
            err.unwrap_custom(),
            (StatusCode::NOT_FOUND, "Document not found".to_string())
        )
    }
}
