use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::node_id::NodeId;

pub struct Document {
    // for bookkeeping
    clock: Hlc,

    // data storage
    minutes_per_ping: Lww<f64>,
}

impl Document {
    pub fn new(node_id: NodeId) -> Self {
        let clock = Hlc::new(node_id);
        let minutes_per_ping = Lww::new(45.0, clock.clone());

        Self {
            clock: clock.next(),
            minutes_per_ping,
        }
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

    #[inline]
    fn check_clock_ordering(&self) {
        // safety property for when we're using more than one CRDT here. Doing
        // this gives us a way to reason about which update happened first, as
        // well as letting us overcome clock drift.
        debug_assert!(&self.clock > self.minutes_per_ping.clock());
    }
}
