use axum::extract::FromRef;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey};
use sqlx::{Pool, Postgres};

/// Shared state needed by requests.
#[derive(Clone, FromRef)]
pub struct State {
    /// Database connection pool.
    pool: Pool<Postgres>,

    /// Key for encoding new JWTs.
    encoding_key: EncodingKey,

    /// Key for verifying existing JWTs.
    decoding_key: DecodingKey,
}

impl State {
    /// Create a new state.
    pub fn new(pool: Pool<Postgres>, jwt_base64_secret: &str) -> Result<Self, Error> {
        Ok(Self {
            pool,
            encoding_key: EncodingKey::from_base64_secret(jwt_base64_secret)?,
            decoding_key: DecodingKey::from_base64_secret(jwt_base64_secret)?,
        })
    }
}
