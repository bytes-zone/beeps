use crate::conn::Conn;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Acquire;

#[derive(Debug, Deserialize)]
pub struct Req {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct Resp {
    email: String,
}

#[tracing::instrument]
pub async fn handler(Conn(mut conn): Conn, Json(req): Json<Req>) -> Result<Json<Resp>, Error> {
    let mut tx = conn.begin().await?;

    // Validation: don't allow a duplicate account if one exists.
    let existing = sqlx::query!(
        "SELECT id FROM accounts WHERE email = $1 LIMIT 1",
        req.email
    )
    .fetch_optional(&mut *tx)
    .await?;

    if existing.is_some() {
        return Err(Error::AlreadyRegistered);
    }

    // We're good, so create the account.
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    sqlx::query!(
        "INSERT INTO accounts (email, password) VALUES ($1, $2)",
        req.email,
        argon2
            .hash_password(req.password.as_bytes(), &salt)?
            .to_string(),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(Resp { email: req.email }))
}

#[derive(Debug, PartialEq)]
pub enum Error {
    AlreadyRegistered,
    InternalError,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            Self::AlreadyRegistered => (
                StatusCode::BAD_REQUEST,
                "An account with this email already exists",
            ),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(?err, "sqlx error");
        Self::InternalError
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        tracing::error!(?err, "error while hashing password");
        Self::InternalError
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::{pool::PoolConnection, query, Postgres};

    #[test_log::test(sqlx::test)]
    async fn test_success(conn: PoolConnection<Postgres>) {
        let email = "test@example.com".to_string();

        let req = Req {
            email: email.clone(),
            password: "test".to_string(),
        };

        let res = handler(Conn(conn), Json(req)).await.unwrap();

        assert_eq!(res.email, email);
    }

    #[test_log::test(sqlx::test)]
    async fn test_duplicate_email(mut conn: PoolConnection<Postgres>) {
        let email = "test@example.com".to_string();

        query("INSERT INTO accounts (email, password) VALUES ($1, 'invalid')")
            .bind(email.clone())
            .execute(&mut *conn)
            .await
            .unwrap();

        let req = Req {
            email,
            password: "test".to_string(),
        };

        let res = handler(Conn(conn), Json(req)).await.unwrap_err();

        assert_eq!(res, Error::AlreadyRegistered);
    }
}
