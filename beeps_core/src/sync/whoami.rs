use serde::{Deserialize, Serialize};

/// Result of calling whoami
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resp {
    /// The email address of the currently logged-in user.
    pub email: String,
}

/// Where the whoami endpoint lives.
pub const PATH: &str = "/api/v1/whoami";
