use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{auth::Claims, error::Error, state::Password};

#[derive(Debug, Deserialize)]
pub struct AuthReq {
    password: String,
    sub: i64,
    document_id: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResp {
    token: String,
    #[serde(rename = "type")]
    type_: &'static str,
}

impl From<String> for AuthResp {
    fn from(token: String) -> Self {
        Self {
            token,
            type_: "Bearer",
        }
    }
}

#[tracing::instrument(skip(encoding_key, password))]
pub async fn handler(
    State(encoding_key): State<EncodingKey>,
    State(password): State<Password>,
    Json(req): Json<AuthReq>,
) -> Result<Json<AuthResp>, Error> {
    if req.password != password.0 {
        return Err(Error::unauthorized("invalid password"));
    }

    if req.document_id < 0 {
        return Err(Error::bad_request("invalid document_id"));
    }

    let now = Utc::now();

    let claims = Claims {
        sub: req.sub,
        iat: now.timestamp(),
        exp: (now + Duration::days(30)).timestamp(),
        document_id: req.document_id,
    };

    let token = encode(&Header::default(), &claims, &encoding_key).map_err(|e| {
        tracing::error!(?e, "error encoding token");
        Error::internal_server_error("failed to encode token")
    })?;

    Ok(Json(AuthResp::from(token)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use jsonwebtoken::EncodingKey;

    fn encoding_key() -> EncodingKey {
        EncodingKey::from_secret(b"secret".as_ref())
    }

    #[tokio::test]
    async fn success() {
        let res = handler(
            State(encoding_key()),
            State(Password("password".to_string())),
            Json(AuthReq {
                password: "password".to_string(),
                sub: 1,
                document_id: 1,
            }),
        )
        .await
        .unwrap();

        assert_eq!(res.0.token.split(".").collect::<Vec<&str>>().len(), 3);
        assert_eq!(res.0.type_, "Bearer");
    }

    #[tokio::test]
    async fn invalid_password() {
        let res = handler(
            State(encoding_key()),
            State(Password("password".to_string())),
            Json(AuthReq {
                password: "HONK".to_string(),
                sub: 1,
                document_id: 1,
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(res.message, "invalid password".to_string());
        assert_eq!(res.status_code, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_document_id() {
        let res = handler(
            State(encoding_key()),
            State(Password("password".to_string())),
            Json(AuthReq {
                password: "password".to_string(),
                sub: 1,
                document_id: -1,
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(res.message, "invalid document_id".to_string());
        assert_eq!(res.status_code, StatusCode::BAD_REQUEST);
    }
}
