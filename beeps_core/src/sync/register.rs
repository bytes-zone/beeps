use super::error::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// The request to register a new account.
#[derive(Debug, Serialize, Deserialize)]
pub struct Req {
    /// Email to use for contact and login.
    pub email: String,

    /// Plaintext password to use for login.
    pub password: String,
}

/// Result of registering a new account.
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {
    /// Email that was successfully registered.
    pub email: String,
}

/// Where the register endpoint lives.
pub const PATH: &str = "/api/v1/register";

/// Register with the server.
pub async fn register(client: reqwest::Client, server: &str, req: Req) -> Result<Resp> {
    let url = Url::parse(server)?.join(PATH)?;

    let resp = client
        .post(url)
        .json(&req)
        .send()
        .await?
        .error_for_status()?;

    Ok(resp.json().await?)
}
