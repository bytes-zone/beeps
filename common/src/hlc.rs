use crate::node_id::NodeId;
use chrono::{DateTime, Utc};
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Hlc {
    timestamp: DateTime<Utc>,
    counter: u64,
    node: NodeId,
}

impl Hlc {
    pub fn new(node: NodeId) -> Self {
        Self {
            timestamp: Utc::now(),
            counter: 0,
            node,
        }
    }

    #[cfg(test)]
    pub fn new_at(node: NodeId, timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            counter: 0,
            node,
        }
    }

    pub fn increment_at(&mut self, now: DateTime<Utc>) {
        if now > self.timestamp {
            self.timestamp = now;
            self.counter = 0;
        } else {
            self.counter += 1;
        }
    }

    pub fn increment(&mut self) {
        self.increment_at(Utc::now());
    }

    pub fn next_at(&self, now: DateTime<Utc>) -> Self {
        let mut next = self.clone();
        next.increment_at(now);
        next
    }

    pub fn next(&self) -> Self {
        self.next_at(Utc::now())
    }

    pub fn mut_receive_at(&mut self, other: &Self, now: DateTime<Utc>) {
        if now > self.timestamp && now > other.timestamp {
            self.timestamp = now;
            self.counter = 0;
            return;
        }

        match self.timestamp.cmp(&other.timestamp) {
            std::cmp::Ordering::Equal => self.counter = self.counter.max(other.counter) + 1,
            std::cmp::Ordering::Greater => self.counter += 1,
            std::cmp::Ordering::Less => {
                self.timestamp = other.timestamp;
                self.counter = other.counter + 1;
            }
        }
    }

    pub fn mut_receive(&mut self, other: &Self) {
        self.mut_receive_at(other, Utc::now());
    }

    pub fn receive_at(&self, other: &Self, now: DateTime<Utc>) -> Self {
        let mut next = self.clone();
        next.mut_receive_at(other, now);
        next
    }

    pub fn receive(&self, other: &Self) -> Self {
        self.receive_at(other, Utc::now())
    }
}

impl Ord for Hlc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.counter.cmp(&other.counter))
            .then(self.node.cmp(&other.node))
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Hlc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.timestamp, self.counter, self.node)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    mod ord {
        use super::*;

        #[test]
        fn timestamp_is_considered_first() {
            let now = Utc::now();

            let hlc1 = Hlc {
                timestamp: now - Duration::seconds(1),
                counter: 0,
                node: NodeId::random(),
            };
            let hlc2 = Hlc {
                timestamp: now + Duration::seconds(1),
                counter: 1,
                node: NodeId::random(),
            };
            assert!(hlc1 < hlc2);
        }

        #[test]
        fn counter_is_considered_second() {
            let now = Utc::now();

            let hlc1 = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::random(),
            };
            let hlc2 = Hlc {
                timestamp: now,
                counter: 1,
                node: NodeId::random(),
            };
            assert!(hlc1 < hlc2);
        }

        #[test]
        fn node_is_considered_third() {
            let now = Utc::now();

            let hlc1 = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::min(),
            };
            let hlc2 = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::max(),
            };
            assert!(hlc1 < hlc2);
        }
    }

    mod next {
        use super::*;

        #[test]
        fn increments_counter_when_timestamp_is_in_the_past() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now + Duration::seconds(1),
                counter: 0,
                node: NodeId::random(),
            };
            let next = hlc.next_at(now);

            // changed
            assert_eq!(next.counter, 1);

            // unchanged
            assert_eq!(next.timestamp, now + Duration::seconds(1));
            assert_eq!(next.node, hlc.node);
        }

        #[test]
        fn increments_timestamp_when_timestamp_is_in_the_future() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now - Duration::seconds(1),
                counter: 1,
                node: NodeId::random(),
            };
            let next = hlc.next_at(now);

            // changed
            assert_eq!(next.timestamp, now);
            assert_eq!(next.counter, 0);

            // unchanged
            assert_eq!(next.node, hlc.node);
        }
    }

    mod receive {
        use super::*;
        #[test]
        fn acts_like_increment_when_both_timestamps_are_behind() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now - Duration::seconds(1),
                counter: 0,
                node: NodeId::random(),
            };
            let other = Hlc {
                timestamp: now - Duration::seconds(1),
                counter: 0,
                node: NodeId::random(),
            };

            let next = hlc.receive_at(&other, now);

            // changed
            assert_eq!(next.timestamp, now, "should accept largest timestamp");
            assert_eq!(next.counter, 0, "should reset counter to 0");

            // unchanged
            assert_eq!(next.node, hlc.node);
        }

        #[test]
        fn increments_counter_when_timestamps_are_equal() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::random(),
            };
            let other = Hlc {
                timestamp: now,
                counter: 1, // should increment from this
                node: NodeId::random(),
            };

            let next = hlc.receive_at(&other, now);

            // changed
            assert_eq!(next.counter, 2, "should increment counter");

            // unchanged
            assert_eq!(next.timestamp, now);
            assert_eq!(next.node, hlc.node);
        }

        #[test]
        fn increments_counter_when_other_is_earlier() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now,
                counter: 1, // should increment from this
                node: NodeId::random(),
            };
            let other = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::random(),
            };

            let next = hlc.receive_at(&other, now);

            // changed
            assert_eq!(next.counter, 2, "should increment counter");

            // unchanged
            assert_eq!(next.timestamp, now);
            assert_eq!(next.node, hlc.node);
        }

        #[test]
        fn accepts_timestamp_and_increments_when_other_timestamp_is_ahead() {
            let now = Utc::now();

            let hlc = Hlc {
                timestamp: now,
                counter: 0,
                node: NodeId::random(),
            };
            let other = Hlc {
                timestamp: now + Duration::seconds(1),
                counter: 1, // should increment from here
                node: NodeId::random(),
            };

            let next = hlc.receive_at(&other, now);

            // changed
            assert_eq!(
                next.timestamp,
                now + Duration::seconds(1),
                "should accept larger timestamp"
            );
            assert_eq!(next.counter, 2, "increments timestamp from other");

            // unchanged
            assert_eq!(next.node, hlc.node);
        }
    }
}
