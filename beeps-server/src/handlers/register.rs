use crate::{conn::Conn, state::AllowRegistration};
use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Acquire;

/// The request to register a new account.
#[derive(Debug, Deserialize)]
pub struct Req {
    /// Email to use for contact and login.
    email: String,

    /// Plaintext password to use for login.
    password: String,
}

/// Result of registering a new account.
#[derive(Debug, Serialize)]
pub struct Resp {
    /// Email that was successfully registered.
    email: String,
}

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    State(allow_registration): State<AllowRegistration>,
    Json(req): Json<Req>,
) -> Result<Json<Resp>, Error> {
    // Validation: don't allow any calls to this endpoint if we don't allow registration.
    if !allow_registration.0 {
        return Err(Error::RegistrationClosed);
    }

    // Validation: don't allow a duplicate account if one exists.
    let mut tx = conn.begin().await?;

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

/// Errors that can occur when registering a new account.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// This server is not configured to allow registration.
    RegistrationClosed,

    /// The email address is already associated with an account.
    AlreadyRegistered,

    /// An internal error occurred.
    InternalError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::RegistrationClosed => (StatusCode::FORBIDDEN, "Registration is closed"),
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

impl From<password_hash::Error> for Error {
    fn from(err: password_hash::Error) -> Self {
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

        let res = handler(Conn(conn), State(AllowRegistration(true)), Json(req))
            .await
            .unwrap();

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

        let res = handler(Conn(conn), State(AllowRegistration(true)), Json(req))
            .await
            .unwrap_err();

        assert_eq!(res, Error::AlreadyRegistered);
    }

    #[test_log::test(sqlx::test)]
    async fn test_registration_closed(conn: PoolConnection<Postgres>) {
        let req = Req {
            email: "test@example.com".to_string(),
            password: "test".to_string(),
        };

        let res = handler(Conn(conn), State(AllowRegistration(false)), Json(req))
            .await
            .unwrap_err();

        assert_eq!(res, Error::RegistrationClosed);
    }
}
