use crate::Document;
use serde::{Deserialize, Serialize};

/// The current document, as seen by the server
#[derive(Debug, Serialize, Deserialize)]
pub struct Resp {
    /// The constructed document
    pub document: Document,
}

/// Where the document push endpoint lives.
pub static PATH: &str = "/api/v1/pull";
