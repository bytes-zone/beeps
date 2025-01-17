use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use beeps_core::sync::push;

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Path(document_id): Path<i64>,
    Json(document): Json<push::Req>,
) -> Result<Json<push::Resp>, Error> {
    // Validate that the user owns the document
    let authed_document = sqlx::query!(
        "SELECT documents.id FROM documents \
        JOIN accounts ON accounts.id = documents.owner_id \
        WHERE accounts.email = $1 AND documents.id = $2",
        claims.sub,
        document_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

    bail_if!(
        authed_document.is_none(),
        "No document with that ID under your account",
        StatusCode::NOT_FOUND
    );

    Ok(Json(push::Resp {}))
}
