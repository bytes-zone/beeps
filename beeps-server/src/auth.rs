use axum::extract::FromRef;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, extract::FromRequestParts};
use axum::{Json, RequestPartsExt};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub iat: i64,
    pub exp: i64,

    // special claims for beeps
    pub document_id: i64,
}

impl Claims {
    #[cfg(test)]
    pub fn test(sub: i64, document_id: i64) -> Self {
        Self {
            sub,
            iat: 0,
            exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp(),
            document_id,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    DecodingKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_ref(state),
            &Validation::default(),
        )
        .map_err(|err| {
            tracing::trace!(?err, "error decoding token");
            AuthError::InvalidToken
        })?;

        Ok(token_data.claims)
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
