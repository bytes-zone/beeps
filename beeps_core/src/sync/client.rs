use super::error::{self, Error};
use super::{login, register, whoami};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;

/// Client for the sync API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    /// The server to connect to. Should only be the protocol and domain, e.g.
    /// `https://beeps.your-domain.com`.
    pub server: String,

    /// Auth. Set this by logging in or registering.
    pub auth: Option<String>,
}

impl Client {
    /// Construct a new client
    pub fn new(server: String) -> Self {
        Self { server, auth: None }
    }

    /// Register with the server.
    ///
    /// ## Errors
    ///
    /// Errors are the same as `handle_response`.
    pub async fn register(
        &self,
        client: &reqwest::Client,
        req: &register::Req,
    ) -> error::Result<register::Resp> {
        let url = Url::parse(&self.server)?.join(register::PATH)?;

        Self::handle_response(client.post(url).json(req)).await
    }

    /// Log into the server.
    ///
    /// ## Errors
    ///
    /// Errors are the same as `handle_response`.
    pub async fn login(
        &self,
        client: &reqwest::Client,
        req: &login::Req,
    ) -> error::Result<login::Resp> {
        let url = Url::parse(&self.server)?.join(login::PATH)?;

        Self::handle_response(client.post(url).json(req)).await
    }

    /// Check that your auth works.
    ///
    /// ## Errors
    ///
    /// Errors are the same as `handle_response`.
    pub async fn whoami(&self, client: &reqwest::Client) -> error::Result<whoami::Resp> {
        let url = Url::parse(&self.server)?.join(whoami::PATH)?;

        self.authenticated(|jwt| client.get(url).bearer_auth(jwt))
            .await
    }

    async fn authenticated<CB, T>(&self, cb: CB) -> Result<T, Error>
    where
        CB: FnOnce(&str) -> reqwest::RequestBuilder,
        T: DeserializeOwned,
    {
        match &self.auth {
            Some(auth) => Self::handle_response(cb(auth)).await,
            None => Err(Error::Client("Unauthorized".to_string())),
        }
    }

    /// Convert an HTTP response into a result, interpreting errors in a
    /// standard way.
    ///
    /// ## Errors
    ///
    /// - `Ok(..)` if the server returned a success (2xx)
    /// - `Error::Client` if the server returned a client error (4xx)
    /// - `Error::Server` if the server returned a server error (5xx)
    /// - `Error::Unexpected` if the server returned something else (the server is
    ///   not supposed to issue redirects or informational responses.)
    async fn handle_response<T>(resp: reqwest::RequestBuilder) -> error::Result<T>
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
}
