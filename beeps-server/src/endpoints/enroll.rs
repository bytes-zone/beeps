use axum::http::{HeaderMap, StatusCode};

use crate::conn::Conn;

pub async fn handler(
    Conn(_): Conn,
    headers: HeaderMap,
) -> Result<String, (StatusCode, &'static str)> {
    let header = headers
        .get("x-temp-account-id")
        .ok_or((StatusCode::BAD_REQUEST, "X-Temp-Account-ID is required"))?;

    match header.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(err) => {
            tracing::warn!(?err, "could not convert header");
            Err((StatusCode::BAD_REQUEST, "Could not read header"))
        }
    }
}
