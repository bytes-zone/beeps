use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::extract::Path;
use axum::Json;
use beeps_core::sync::document_push;

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Path(document_id): Path<i64>,
    Json(document): Json<document_push::Req>,
) -> Result<Json<document_push::Resp>, Error> {
    Ok(Json(document_push::Resp {}))
}
