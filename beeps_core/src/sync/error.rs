use thiserror::Error;

/// Easy alias for error handling
pub type Result<T> = std::result::Result<T, Error>;

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
}
