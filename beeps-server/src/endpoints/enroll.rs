use crate::auth::Claims;
use crate::conn::Conn;
use axum::http::StatusCode;
use sqlx::query;

#[tracing::instrument]
pub async fn handler(
    claims: Claims,
    Conn(mut conn): Conn,
) -> Result<String, (StatusCode, &'static str)> {
    let aggregate = query!(
        "SELECT MAX(node) FROM operations WHERE document_id = $1",
        claims.document_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| {
        tracing::error!(?err, "error querying");
        (StatusCode::INTERNAL_SERVER_ERROR, "error querying")
    })?;

    Ok(aggregate.max.unwrap_or(0).to_string())
}
