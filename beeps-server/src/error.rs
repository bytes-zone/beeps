use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub status_code: StatusCode,
}

impl Error {
    pub fn new(message: &str, status_code: StatusCode) -> Self {
        Self {
            message: message.to_string(),
            status_code,
        }
    }
    pub fn bad_request(message: &str) -> Self {
        Self::new(message, StatusCode::BAD_REQUEST)
    }

    pub(crate) fn internal_server_error(message: &str) -> Self {
        Self::new(message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub(crate) fn unauthorized(message: &str) -> Self {
        Self::new(message, StatusCode::UNAUTHORIZED)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        let body = Json(serde_json::json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
    }
}
