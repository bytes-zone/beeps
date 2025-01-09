use crate::error::Error;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Claims a JWT can make in our system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    #[expect(clippy::absolute_paths)]
    fn from_str(
        token: &str,
        decoding_key: &DecodingKey,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        decode::<Self>(token, decoding_key, &Validation::default()).map(|data| data.claims)
    }
}

/// Issue a new JWT with the given subject and document ID
pub fn issue(
    encoding_key: &EncodingKey,
    sub: &str,
    document_id: i64,
) -> jsonwebtoken::errors::Result<String> {
    let claims = Claims {
        sub: sub.to_string(),
        iat: 0,
        exp: (chrono::Utc::now() + chrono::Duration::days(90)).timestamp(),
        document_id,
    };

    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, encoding_key)
}

impl<S> FromRequestParts<S> for Claims
where
    DecodingKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::custom("missing or invalid authorization header"))?;

        Claims::from_str(bearer.token(), &DecodingKey::from_ref(state))
            .map_err(|_| Error::custom("invalid token"))
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

        assert_eq!(
            result.unwrap_err().kind(),
            &jsonwebtoken::errors::ErrorKind::ExpiredSignature
        );
    }
}
