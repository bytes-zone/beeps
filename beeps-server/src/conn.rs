use crate::error::Error;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::{pool::PoolConnection, PgPool};

/// A connection to the database
pub struct Conn(pub PoolConnection<sqlx::Postgres>);

impl<State> FromRequestParts<State> for Conn
where
    PgPool: FromRef<State>,
    State: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await?;

        Ok(Self(conn))
    }
}
