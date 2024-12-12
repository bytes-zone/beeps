use crate::grow_only_map::GrowOnlyMap;
use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::node_id::NodeId;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Document {
    // for bookkeeping
    clock: Hlc,

    // data storage
    minutes_per_ping: Lww<f64>,
    pings: GrowOnlyMap<DateTime<Utc>, Lww<Option<String>>>,
}

impl Document {
    pub fn new(node_id: NodeId) -> Self {
        let clock = Hlc::new(node_id);
        let minutes_per_ping = Lww::new(45.0, clock.clone());

        let out = Self {
            clock: clock.next(),
            minutes_per_ping,
            pings: GrowOnlyMap::new(),
        };

        out.check_clock_ordering();

        out
    }

    fn next_clock(&mut self) -> Hlc {
        self.clock.increment();
        self.clock.clone()
    }

    pub fn minutes_per_ping(&self) -> &f64 {
        self.minutes_per_ping.value()
    }

    pub fn set_minutes_per_ping(&mut self, new: f64) {
        let clock = self.next_clock();
        self.minutes_per_ping.set(new, clock);
        self.check_clock_ordering();
    }

    pub fn pings(&self) -> &GrowOnlyMap<DateTime<Utc>, Lww<Option<String>>> {
        &self.pings
    }

    pub fn add_ping(&mut self, when: DateTime<Utc>) {
        let clock = self.next_clock();
        self.pings.insert(when, Lww::new(None, clock));
        self.check_clock_ordering();
    }

    pub fn tag_ping(&mut self, when: DateTime<Utc>, tag: String) {
        let clock = self.next_clock();
        self.pings.insert(when, Lww::new(Some(tag), clock));
        self.check_clock_ordering();
    }

    #[inline]
    fn check_clock_ordering(&self) {
        // safety property for when we're using more than one CRDT here. Doing
        // this gives us a way to reason about which update happened first, as
        // well as letting us overcome clock drift.
        debug_assert!(
            &self.clock >= self.minutes_per_ping.clock(),
            "{} < {}",
            self.clock,
            self.minutes_per_ping.clock()
        );
        for (_, lww) in self.pings.iter() {
            debug_assert!(
                &self.clock >= lww.clock(),
                "{} < {}",
                self.clock,
                self.minutes_per_ping.clock()
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn minutes_per_ping() {
        let node_id = NodeId::random();
        let mut doc = Document::new(node_id);

        doc.set_minutes_per_ping(60.0);
        assert_eq!(*doc.minutes_per_ping(), 60.0);
    }

    #[test]
    fn add_ping() {
        let node_id = NodeId::random();
        let mut doc = Document::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        assert_eq!(doc.pings().get(&when).map(|lww| lww.value()), Some(&None));
    }

    #[test]
    fn set_ping() {
        let node_id = NodeId::random();
        let mut doc = Document::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        doc.tag_ping(when, "test".to_string());
        assert_eq!(
            doc.pings().get(&when).and_then(|lww| lww.value().clone()),
            Some("test".to_string())
        );
    }
}
