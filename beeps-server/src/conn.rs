use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use sqlx::{pool::PoolConnection, PgPool};

/// A connection to the database
pub struct Conn(pub PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<State> FromRequestParts<State> for Conn
where
    PgPool: FromRef<State>,
    State: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool
            .acquire()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        Ok(Self(conn))
    }
}
