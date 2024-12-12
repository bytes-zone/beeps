use crate::gmap::GMap;
use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::merge::Merge;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    pub minutes_per_ping: Lww<f64>,
    pub pings: GMap<DateTime<Utc>, Lww<Option<String>>>,
}

impl State {
    pub fn new_at(clock: Hlc) -> Self {
        Self {
            minutes_per_ping: Lww::new(45.0, clock.clone()),
            pings: GMap::new(),
        }
    }
}

impl Merge for State {
    fn merge(mut self, other: Self) -> Self {
        self.minutes_per_ping = self.minutes_per_ping.merge(other.minutes_per_ping);
        self.pings = self.pings.merge(other.pings);

        self
    }
}
