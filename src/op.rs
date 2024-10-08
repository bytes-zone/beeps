use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Op {
    // Pings
    AddPing { when: DateTime<Utc> },

    // Tags
    SetTag { when: DateTime<Utc>, tag: String },

    // Settings
    SetLambda { lambda: f64 },
}
