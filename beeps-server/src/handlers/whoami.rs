use crate::jwt::Claims;
use axum::Json;
use beeps_core::sync::whoami;

#[tracing::instrument]
pub async fn handler(claims: Claims) -> Json<whoami::Resp> {
    Json(whoami::Resp { email: claims.sub })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_success() {
        let claims = Claims {
            sub: "test@example.com".to_string(),
            iat: 0,
            exp: 1,
            document_id: 2,
        };

        let Json(resp) = handler(claims.clone()).await;

        assert_eq!(resp, whoami::Resp { email: claims.sub });
    }
}
