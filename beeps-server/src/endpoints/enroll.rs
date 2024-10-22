use axum::http::{HeaderMap, StatusCode};
use sqlx::query;

use crate::conn::Conn;

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    headers: HeaderMap,
) -> Result<String, (StatusCode, &'static str)> {
    let header = headers
        .get("x-temp-account-id")
        .ok_or((StatusCode::BAD_REQUEST, "X-Temp-Account-ID is required"))
        .and_then(|h| {
            h.to_str().map_err(|err| {
                tracing::warn!(?err, "could not convert header");
                (StatusCode::BAD_REQUEST, "Could not read header")
            })
        })
        .and_then(|s| {
            s.parse::<i64>()
                .map_err(|_| (StatusCode::BAD_REQUEST, "Needed an integer ID"))
        })?;

    let aggregate = query!(
        "SELECT MAX(node) FROM operations WHERE document_id = $1",
        header
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| {
        tracing::error!(?err, "error querying");
        (StatusCode::INTERNAL_SERVER_ERROR, "error querying")
    })?;

    Ok(aggregate.max.unwrap_or(0).to_string())
}
