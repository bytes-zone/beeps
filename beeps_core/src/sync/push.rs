use crate::Document;
use serde::{Deserialize, Serialize};

/// The replica data we send to the server.
pub type Req = Document;

/// Confirmation that the server accepted the document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {}

/// Where the document push endpoint lives.
pub static PATH: &str = "/api/v1/push/:id";

/// Construct a path given a document ID.
pub fn path(id: i64) -> String {
    PATH.replace(":id", &id.to_string())
}
