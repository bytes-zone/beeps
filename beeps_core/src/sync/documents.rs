use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of calling documents
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resp {
    /// The list of documents associated with this account
    pub documents: Vec<Document>,
}

/// A single document that the user can select.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    /// The ID of the document for use in later calls
    pub id: i64,

    /// When the document was created, to differentiate between documents.
    pub created_at: DateTime<Utc>,

    /// When the document was last updated, to differentiate between documents.
    pub updated_at: DateTime<Utc>,
}

/// Where the documents endpoint lives.
pub const PATH: &str = "/api/v1/documents";
