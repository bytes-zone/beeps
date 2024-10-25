use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{auth::Claims, error::Error};

#[derive(Debug, Deserialize)]
pub struct AuthReq {
    sub: usize,
    document_id: i64,
}

#[derive(Serialize)]
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

#[tracing::instrument(skip(encoding_key))]
pub async fn handler(
    State(encoding_key): State<EncodingKey>,
    Json(req): Json<AuthReq>,
) -> Result<Json<AuthResp>, Error> {
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
