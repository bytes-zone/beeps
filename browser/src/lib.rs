mod utils;

use common::node_id::NodeId;
use wasm_bindgen::prelude::*;

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();

    // At the moment, this only references code we care about to make sure it's
    // included in the bundle (and therefore can compile with wasm.) It's not
    // intended to demonstrate any particular thing.
    alert("Beginning test.");

    let clock = common::hlc::Hlc::new(NodeId::random());
    let lww = common::lww::Lww::new(1, clock);

    let mut map = common::grow_only_map::GrowOnlyMap::new();
    map.insert("test", lww);

    alert("set value to…");
    alert(&map.get(&"test").unwrap().value().to_string());
}
