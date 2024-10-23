use axum::extract::FromRef;
use sqlx::{Pool, Postgres};

#[derive(Clone, FromRef)]
pub struct State {
    pool: Pool<Postgres>,
}

impl State {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}
