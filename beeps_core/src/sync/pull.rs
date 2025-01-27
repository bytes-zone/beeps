use crate::Document;

/// The current document, as seen by the server
pub type Resp = Document;

/// Where the document push endpoint lives.
pub static PATH: &str = "/api/v1/pull";
