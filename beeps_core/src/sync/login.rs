use serde::{Deserialize, Serialize};

/// The request to log into the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Req {
    /// Email to use for contact and login.
    pub email: String,

    /// Plaintext password to use for login.
    pub password: String,
}

/// Result of logging in.
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {
    /// JWT to use for future requests.
    pub jwt: String,
}

/// Where the login endpoint lives.
pub const PATH: &str = "/api/v1/login";
