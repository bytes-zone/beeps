use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::sync::pull;
use beeps_core::Document;

#[tracing::instrument]
pub async fn handler(Conn(conn): Conn, claims: Claims) -> Result<Json<pull::Resp>, Error> {
    let doc = Document::default();

    Ok(Json(doc))
}

#[cfg(test)]
mod test {}
