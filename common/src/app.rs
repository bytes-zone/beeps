use crate::hlc::Hlc;
use crate::lww_map::LwwMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Store {
    clock: Hlc,
    pings: LwwMap<DateTime<Utc>, Option<String>>,
}

impl Store {
    pub fn new(node: Uuid) -> Self {
        Self {
            clock: Hlc::new(node),
            pings: LwwMap::new(),
        }
    }

    /// Check consistency of the store. Specifically: the top-level clock should
    /// always be higher than any other clock.
    pub fn self_check(&self) -> bool {
        self.pings.iter().all(|(_, lww)| *lww.clock() < self.clock)
    }
}
