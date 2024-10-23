use crate::conn::Conn;
use crate::response::internal_error;
use axum::response::IntoResponse;
use sqlx::{query, Postgres};

pub async fn handler(Conn(mut conn): Conn) -> impl IntoResponse {
    let q = query!("select * from accounts")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)?;

    tracing::warn!(?q, "sample query");

    sqlx::query_scalar::<Postgres, String>("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
}
