use argon2::password_hash;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// An error from the API
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Something went wrong which we should log but not expose to clients.
    Internal,

    /// Some handler-specific error
    Custom(StatusCode, String),
}

/// Return an error from a handler-specific error type.
#[macro_export]
macro_rules! bail {
    ($message:expr) => {
        return Err($crate::error::Error::Custom(
            axum::http::StatusCode::BAD_REQUEST,
            $message,
        ))
    };
    ($status:expr, $message:expr) => {
        return Err($crate::error::Error::Custom($status, $message))
    };
}

/// `bail!` conditionally.
#[macro_export]
macro_rules! bail_if {
    ($cond:expr, $message:expr) => {
        if $cond {
            bail!($message);
        }
    };
    ($cond:expr, $status:expr, $message:expr) => {
        if $cond {
            bail!($status, $message);
        }
    };
}

impl Error {
    /// Unwrap a handler-specific error
    #[cfg(test)]
    pub fn unwrap_custom(self) -> (StatusCode, String) {
        match self {
            Self::Custom(status_code, message) => (status_code, message),
            Self::Internal => panic!("called `Error::unwrap_handler` on an `Internal`"),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(?err, "sqlx error");
        Self::Internal
    }
}

impl From<password_hash::Error> for Error {
    fn from(err: password_hash::Error) -> Self {
        tracing::error!(?err, "password hashing error");
        Self::Internal
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            Self::Custom(status_code, message) => (status_code, message),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
