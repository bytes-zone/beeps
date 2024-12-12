use crate::gmap::GMap;
use crate::hlc::Hlc;
use crate::lww::Lww;
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
