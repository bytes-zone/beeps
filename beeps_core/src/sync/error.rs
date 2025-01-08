use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::result;
use thiserror::Error;

/// Easy alias for error handling
pub type Result<T> = result::Result<T, Error>;

/// Errors that can happen while processing requests
#[derive(Debug, Error)]
pub enum Error {
    /// We couldn't parse a URL, for example if the base URL was invalid.
    #[error("URL error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// We encountered an HTTP error, for example if the server returned a 404
    /// or 500.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// There was an error on the client side, for example a missing or expired
    /// token or a validation failure.
    #[error("Client error: {0}")]
    Client(String),

    /// There was an error on the server
    #[error("Server error")]
    Server,

    /// The server sent us something unexpected
    #[error("The server sent an unexpected response")]
    Unexpected(StatusCode),
}

#[expect(clippy::module_name_repetitions)]
/// An error response from the server
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResp {
    /// The error message
    pub error: String,
}
