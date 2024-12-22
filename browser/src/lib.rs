//! Browser interface for beeps storage, compiled to WASM.

#[expect(clippy::missing_docs_in_private_items)]
mod utils;

use beeps_core::{NodeId, Replica};
use chrono::Utc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

/// At the moment, this only references code we care about to make sure it's
/// included in the bundle (and therefore can compile with wasm.) It's not
/// intended to demonstrate any particular thing.
///
/// # Panics
///
/// Does some silly things to make sure all the code is included in the bundle
/// so we can see how big we're getting.
#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();

    alert("Beginning test.");

    let mut replica = Replica::new(NodeId::random());
    replica.set_minutes_per_ping(45);

    let now = Utc::now();
    replica.add_ping(now);
    replica.tag_ping(now, Some("HI!".to_string()));

    alert(&replica.state().pings.contains(&now).to_string());
    alert(replica.state().get_tag(&now).unwrap());
}
