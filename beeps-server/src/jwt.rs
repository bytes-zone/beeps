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

/// Claims a JWT can make in our system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Claims {
    /// The subject of the claims. In our case, the email address associated
    /// with the account.
    pub sub: String,

    /// When the token was issued.
    pub iat: i64,

    /// When the token expires.
    pub exp: i64,

    /// What document ID this token grants access to.
    pub document_id: i64,
}

impl Claims {
    #[cfg(test)]
    pub fn test(sub: &str, document_id: i64) -> Self {
        Self {
            sub: sub.to_string(),
            iat: 0,
            exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp(),
            document_id,
        }
    }

    /// Parse and verify a token from a string
    fn from_str(token: &str, decoding_key: &DecodingKey) -> Result<Self, AuthError> {
        decode::<Self>(token, decoding_key, &Validation::default())
            .map_err(|err| {
                tracing::trace!(?err, "error decoding token");
                AuthError::InvalidToken
            })
            .map(|data| data.claims)
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

        Claims::from_str(bearer.token(), &DecodingKey::from_ref(state))
    }
}

/// Errors returned with JWT auth fails
#[derive(Debug, PartialEq)]
pub enum AuthError {
    /// The token itself was invalid (expired, improperly signed, etc)
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

#[cfg(test)]
mod test {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey};

    #[test]
    fn valid_token() {
        let claims = Claims::test("test@example.com", 1);
        let key = EncodingKey::from_secret(b"secret");
        let token = encode(&jsonwebtoken::Header::default(), &claims, &key).unwrap();

        let decoding_key = DecodingKey::from_secret(b"secret");
        let result = Claims::from_str(&token, &decoding_key);
        assert_eq!(result.unwrap(), claims);
    }

    #[test]
    fn test_expired_token() {
        let claims = Claims {
            sub: "test@example.com".to_string(),
            iat: 0,
            exp: 0,
            document_id: 1,
        };
        let key = EncodingKey::from_secret(b"secret");
        let token = encode(&jsonwebtoken::Header::default(), &claims, &key).unwrap();

        let decoding_key = DecodingKey::from_secret(b"secret");
        let result = Claims::from_str(&token, &decoding_key);
        assert_eq!(result.unwrap_err(), AuthError::InvalidToken);
    }
}
