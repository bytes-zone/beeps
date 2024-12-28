use axum::extract::FromRef;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey};
use sqlx::{Pool, Postgres};

#[derive(Clone, FromRef)]
pub struct State {
    pool: Pool<Postgres>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl State {
    pub fn new(pool: Pool<Postgres>, jwt_base64_secret: &str) -> Result<Self, Error> {
        Ok(Self {
            pool,
            encoding_key: EncodingKey::from_base64_secret(jwt_base64_secret)?,
            decoding_key: DecodingKey::from_base64_secret(jwt_base64_secret)?,
        })
    }
}
