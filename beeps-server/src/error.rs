use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

pub struct Error {
    message: String,
    status_code: StatusCode,
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
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        let body = Json(serde_json::json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
    }
}
