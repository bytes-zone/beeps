use crate::grow_only_map::GrowOnlyMap;
use crate::{hlc::Hlc, lww::Lww};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Store {
    clock: Hlc,
    pings: GrowOnlyMap<DateTime<Utc>, Lww<Option<String>>>,
}

impl Store {
    pub fn new(node: Uuid) -> Self {
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
