use beeps_core::Document;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct UiDocument {
    pings: Vec<PingWithTag>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct PingWithTag {
    ping: DateTime<Utc>,
    tag: Option<String>,
}

impl From<&Document> for UiDocument {
    fn from(doc: &Document) -> Self {
        let mut pings: Vec<PingWithTag> = doc
            .pings
            .iter()
            .map(|ping| PingWithTag {
                ping: ping.clone(),
                tag: doc.get_tag(ping).cloned(),
            })
            .collect();

        // We store pings oldest-to-newest, but want to present them
        // newest-to-oldest in the UI.
        pings.reverse();

        Self { pings }
    }
}
