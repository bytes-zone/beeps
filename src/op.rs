use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Op {
    // Pings
    AddPing {
        when: DateTime<Utc>,
    },
    SetOffset {
        when: DateTime<Utc>,
        offset: i32,
    },

    // Tags
    SetTag {
        when: DateTime<Utc>,
        tag: String,
    },
    SetExtra {
        when: DateTime<Utc>,
        key: String,
        value: String,
    },
    UnsetExtra {
        when: DateTime<Utc>,
        key: String,
    },

    // Settings
    SetLambda {
        lambda: f64,
    },
}
