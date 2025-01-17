use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::extract::Path;
use axum::Json;
use beeps_core::sync::push;

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Path(document_id): Path<i64>,
    Json(document): Json<push::Req>,
) -> Result<Json<push::Resp>, Error> {
    Ok(Json(push::Resp {}))
}
