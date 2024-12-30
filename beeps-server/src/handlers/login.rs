use crate::bail;
use crate::conn::Conn;
use crate::error::Error;
use argon2::{password_hash, Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use serde::{Deserialize, Serialize};

/// This should be the same for both missing accounts and incorrect passwords so
/// as not to give additional information about what accounts exist to someone
/// probing the system.
static BAD_LOGIN_MESSAGE: &str = "incorrect email or password";

#[derive(Debug, Deserialize)]
pub struct Req {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct Resp {
    jwt: String,
}

#[tracing::instrument(skip(req))]
pub async fn handler(Conn(mut conn): Conn, Json(req): Json<Req>) -> Result<Json<Resp>, Error> {
    let account = sqlx::query!(
        "SELECT email, password FROM accounts WHERE email = $1 LIMIT 1",
        req.email
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(Error::custom(BAD_LOGIN_MESSAGE))?;

    let hash = PasswordHash::new(&account.password)?;

    if let Err(err) = Argon2::default().verify_password(req.password.as_bytes(), &hash) {
        if err == password_hash::Error::Password {
            bail!(BAD_LOGIN_MESSAGE)
        }

        tracing::error!(?err, "error verifying password");
        return Err(Error::Internal);
    }

    Ok(Json(Resp {
        jwt: "hello".to_string(),
    }))
}
