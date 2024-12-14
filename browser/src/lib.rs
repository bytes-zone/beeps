#![warn(
    missing_docs,
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::absolute_paths,
    clippy::alloc_instead_of_core,
    clippy::decimal_literal_representation,
    clippy::missing_docs_in_private_items
)]
#![allow(clippy::must_use_candidate)]

//! Browser interface for beeps storage, compiled to WASM.

#[expect(clippy::missing_docs_in_private_items)]
mod utils;

use chrono::Utc;
use common::node_id::NodeId;
use common::replica::Replica;
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
    replica.tag_ping(now, "HI!".to_string());

    alert(
        replica
            .state()
            .pings
            .get(&now)
            .unwrap()
            .value()
            .as_ref()
            .unwrap(),
    );
}
