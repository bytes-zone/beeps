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
    /// JWT to use for future requests.
    pub jwt: String,
}

/// Where the register endpoint lives.
pub const PATH: &str = "/api/v1/register";
