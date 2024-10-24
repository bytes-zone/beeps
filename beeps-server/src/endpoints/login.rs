use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::auth::Claims;

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

#[tracing::instrument(skip(encoding_key))]
pub async fn handler(
    State(encoding_key): State<EncodingKey>,
    Json(req): Json<AuthReq>,
) -> Result<Json<AuthResp>, (StatusCode, &'static str)> {
    if req.document_id < 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid document_id"));
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
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode token")
    })?;

    Ok(Json(AuthResp {
        token,
        type_: "Bearer",
    }))
}
