use crate::Document;
use serde::{Deserialize, Serialize};

/// The replica data we send to the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Req {
    /// Which document we're pushing.
    pub document_id: i64,

    /// The document contents to push.
    pub document: Document,
}

/// Confirmation that the server accepted the document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {}

/// Where the document push endpoint lives.
pub static PATH: &str = "/api/v1/push";
