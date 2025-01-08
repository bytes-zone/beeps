/// Things that can go wrong in the API
pub mod error;
pub use error::Error;

/// Register with the server
pub mod register;
pub use register::register;

use serde::de::DeserializeOwned;

/// Convert an HTTP response into a result, interpreting errors in the
/// standard way.
///
/// ## Errors
///
/// - `Ok(..)` if the server returned a success (2xx)
/// - `Error::Client` if the server returned a client error (4xx)
/// - `Error::Server` if the server returned a server error (5xx)
/// - `Error::Unexpected` if the server returned something else (the server is
///   not supposed to issue redirects or informational responses.)
pub async fn handle_response<T>(resp: reqwest::RequestBuilder) -> error::Result<T>
where
    T: DeserializeOwned,
{
    let resp = resp.send().await?;

    let status = resp.status();

    if status.is_success() {
        Ok(resp.json().await?)
    } else if status.is_client_error() {
        let err: error::ErrorResp = resp.json().await?;
        Err(Error::Client(err.error))
    } else if status.is_server_error() {
        Err(Error::Server)
    } else {
        Err(Error::Unexpected(status))
    }
}
