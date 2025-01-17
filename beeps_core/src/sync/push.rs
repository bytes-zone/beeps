use crate::Replica;
use serde::{Deserialize, Serialize};

/// The replica data we send to the server.
pub type Req = Replica;

/// Confirmation that the server accepted the document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {}

/// Where the document push endpoint lives.
pub static PATH: &str = "/api/v1/documents/:id";

/// Make a path with the ID in the correct segment
pub fn make_path(id: i64) -> String {
    PATH.replace(":id", &id.to_string())
}
