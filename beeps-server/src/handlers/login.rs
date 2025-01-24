use crate::bail;
use crate::conn::Conn;
use crate::error::Error;
use crate::jwt;
use argon2::{password_hash, Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, Json};
use beeps_core::sync::login::{Req, Resp};
use jsonwebtoken::EncodingKey;
use sqlx::query;

/// This should be the same for both missing accounts and incorrect passwords so
/// as not to give additional information about what accounts exist to someone
/// probing the system.
static BAD_LOGIN_MESSAGE: &str = "incorrect email or password";

#[tracing::instrument(skip(conn, req, encoding_key), fields(req.email = %req.email))]
pub async fn handler(
    Conn(mut conn): Conn,
    State(encoding_key): State<EncodingKey>,
    Json(req): Json<Req>,
) -> Result<Json<Resp>, Error> {
    let account = sqlx::query!(
        "SELECT id, email, password FROM accounts WHERE email = $1 LIMIT 1",
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

    let document = query!(
        "SELECT id FROM documents WHERE owner_id = $1 LIMIT 1",
        account.id,
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(Json(Resp {
        jwt: jwt::issue(&encoding_key, &account.email, document.id)?,
    }))
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use jsonwebtoken::{DecodingKey, Validation};
    use jwt::Claims;
    use sqlx::{pool::PoolConnection, Postgres};

    use crate::handlers::test::TestDoc;

    use super::*;

    fn encoding_key() -> EncodingKey {
        EncodingKey::from_secret(b"secret".as_ref())
    }

    fn decoding_key() -> DecodingKey {
        DecodingKey::from_secret(b"secret".as_ref())
    }

    #[test_log::test(sqlx::test)]
    async fn test_success(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let resp = handler(
            Conn(conn),
            State(encoding_key()),
            Json(Req {
                email: doc.email.clone(),
                password: doc.password.clone(),
            }),
        )
        .await
        .unwrap();

        let token =
            jsonwebtoken::decode::<Claims>(&resp.jwt, &decoding_key(), &Validation::default())
                .unwrap();

        assert_eq!(token.claims.sub, doc.email);
    }

    #[test_log::test(sqlx::test)]
    async fn test_bad_email(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let resp = handler(
            Conn(conn),
            State(encoding_key()),
            Json(Req {
                email: "honk@example.com".to_string(),
                password: doc.password.clone(),
            }),
        )
        .await
        .unwrap_err()
        .unwrap_custom();

        assert_eq!(
            resp,
            (StatusCode::BAD_REQUEST, BAD_LOGIN_MESSAGE.to_string())
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_bad_password(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let resp = handler(
            Conn(conn),
            State(encoding_key()),
            Json(Req {
                email: doc.email.clone(),
                password: "bad password".to_string(),
            }),
        )
        .await
        .unwrap_err()
        .unwrap_custom();

        assert_eq!(
            resp,
            (StatusCode::BAD_REQUEST, BAD_LOGIN_MESSAGE.to_string())
        );
    }
}
