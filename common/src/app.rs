use crate::grow_only_map::GrowOnlyMap;
use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::node_id::NodeId;
use chrono::{DateTime, Utc};

pub struct Store {
    clock: Hlc,
    pings: GrowOnlyMap<DateTime<Utc>, Lww<Option<String>>>,
}

impl Store {
    pub fn new(node: NodeId) -> Self {
        Self {
            clock: Hlc::new(node),
            pings: GrowOnlyMap::new(),
        }
    }

    /// Check consistency of the store. Specifically: the top-level clock should
    /// always be higher than any other clock.
    pub fn self_check(&self) -> bool {
        self.pings.iter().all(|(_, lww)| *lww.clock() < self.clock)
    }
}
