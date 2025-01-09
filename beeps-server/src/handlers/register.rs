use crate::bail_if;
use crate::error::Error;
use crate::jwt;
use crate::state::AllowRegistration;
use crate::{bail, conn::Conn};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use beeps_core::sync::register::{Req, Resp};
use jsonwebtoken::EncodingKey;
use sqlx::Acquire;

#[tracing::instrument(skip(conn, encoding_key, req), fields(req.email = %req.email))]
pub async fn handler(
    Conn(mut conn): Conn,
    State(AllowRegistration(allow_registration)): State<AllowRegistration>,
    State(encoding_key): State<EncodingKey>,
    Json(req): Json<Req>,
) -> Result<Json<Resp>, Error> {
    // Validation: don't allow any calls to this endpoint if we don't allow registration.
    bail_if!(
        !allow_registration,
        "Registration is closed",
        StatusCode::FORBIDDEN
    );

    // Validation: don't allow a duplicate account if one exists.
    let mut tx = conn.begin().await?;

    let existing = sqlx::query!(
        "SELECT id FROM accounts WHERE email = $1 LIMIT 1",
        req.email
    )
    .fetch_optional(&mut *tx)
    .await?;

    bail_if!(
        existing.is_some(),
        "An account with this email already exists"
    );

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

    Ok(Json(Resp {
        jwt: jwt::issue(
            &encoding_key,
            &req.email,
            0, // TODO
        )?,
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::jwt::Claims;
    use jsonwebtoken::{DecodingKey, Validation};
    use sqlx::{pool::PoolConnection, query, Postgres};

    fn encoding_key() -> EncodingKey {
        EncodingKey::from_secret(b"secret".as_ref())
    }

    fn decoding_key() -> DecodingKey {
        DecodingKey::from_secret(b"secret".as_ref())
    }

    #[test_log::test(sqlx::test)]
    async fn test_success(conn: PoolConnection<Postgres>) {
        let email = "test@example.com".to_string();

        let req = Req {
            email: email.clone(),
            password: "test".to_string(),
        };

        let resp = handler(
            Conn(conn),
            State(AllowRegistration(true)),
            State(encoding_key()),
            Json(req),
        )
        .await
        .unwrap();

        let token =
            jsonwebtoken::decode::<Claims>(&resp.jwt, &decoding_key(), &Validation::default())
                .unwrap();

        assert_eq!(token.claims.sub, email);
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

        let res = handler(
            Conn(conn),
            State(AllowRegistration(true)),
            State(encoding_key()),
            Json(req),
        )
        .await
        .unwrap_err()
        .unwrap_custom();

        assert_eq!(
            res,
            (
                StatusCode::BAD_REQUEST,
                "An account with this email already exists".to_string()
            )
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_registration_closed(conn: PoolConnection<Postgres>) {
        let req = Req {
            email: "test@example.com".to_string(),
            password: "test".to_string(),
        };

        let res = handler(
            Conn(conn),
            State(AllowRegistration(false)),
            State(encoding_key()),
            Json(req),
        )
        .await
        .unwrap_err()
        .unwrap_custom();

        assert_eq!(
            res,
            (StatusCode::FORBIDDEN, "Registration is closed".to_string())
        );
    }
}
