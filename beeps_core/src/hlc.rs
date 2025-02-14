use crate::node_id::NodeId;
use chrono::{DateTime, TimeZone, Utc};
use std::cmp::Ordering;
use std::fmt::{self, Display};

/// A Hybrid Logical Clock (HLC.) Builds on a Lamport clock by adding a
/// timestamp. This allows us to get a monotonically-increasing clock despite
/// the fact that wall time can go backwards or smear for leap seconds.
#[derive(PartialEq, Eq, Copy, Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Hlc {
    /// The physical time component. First to break ties.
    #[cfg_attr(test, proptest(strategy = "crate::test::timestamp()"))]
    timestamp: DateTime<Utc>,

    /// A counter we increment if the new physical time is equal to or less than
    /// the previous. Second to break ties.
    #[cfg_attr(test, proptest(strategy = "0..=10u16"))]
    counter: u16,

    /// The node ID of the replica that generated this HLC. Third to break ties.
    node: NodeId,
}

impl Hlc {
    /// Create a new HLC with the given node ID.
    pub fn new(node: NodeId) -> Self {
        Self {
            timestamp: Utc::now(),
            counter: 0,
            node,
        }
    }

    /// Create a new HLC with the given node ID and timestamp. Only for testing
    /// and loading data from the database. Otherwise always use `.next` or
    /// `.increment` to avoid creating timestamps in the past.
    pub fn new_at(node: NodeId, timestamp: DateTime<Utc>, counter: u16) -> Self {
        Self {
            timestamp,
            counter,
            node,
        }
    }

    /// An HCL less than any other HLC. Useful as a base or default value in
    /// something like an LWW-Register.
    pub fn zero() -> Self {
        Self {
            timestamp: Utc.timestamp_opt(0, 0).unwrap(),
            counter: 0,
            node: NodeId::min(),
        }
    }

    /// Increment as if it's a specific time.
    ///
    /// TODO: might be better to be private.
    pub fn increment_at(&mut self, now: DateTime<Utc>) {
        if now > self.timestamp {
            self.timestamp = now;
            self.counter = 0;
        } else {
            self.counter += 1;
        }
    }

    /// Increment this HLC at the current physical time.
    pub fn increment(&mut self) {
        self.increment_at(Utc::now());
    }

    /// Get the next HLC as if it's a specific time.
    ///
    /// TODO: might be better to be private.
    #[must_use]
    pub fn next_at(&self, now: DateTime<Utc>) -> Self {
        let mut next = *self;
        next.increment_at(now);
        next
    }

    /// Get the next HLC at the current physical time.
    #[must_use]
    pub fn next(&self) -> Self {
        self.next_at(Utc::now())
    }

    /// Update this HCL to be higher than a HLC we're receiving from another
    /// replica. This is helpful for being able to continue to issue timestamps
    /// across all replicas, even if some physical clocks are rushing.
    ///
    /// This variant allows you to specify what time "now" is.
    pub fn mut_receive_at(&mut self, other: &Self, now: DateTime<Utc>) {
        if now > self.timestamp && now > other.timestamp {
            self.timestamp = now;
            self.counter = 0;
            return;
        }

        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => self.counter = self.counter.max(other.counter) + 1,
            Ordering::Greater => self.counter += 1,
            Ordering::Less => {
                self.timestamp = other.timestamp;
                self.counter = other.counter + 1;
            }
        }
    }

    /// Update this HCL to be higher than a HLC we're receiving from another
    /// replica. This is helpful for being able to continue to issue timestamps
    /// across all replicas, even if some physical clocks are rushing.
    pub fn mut_receive(&mut self, other: &Self) {
        self.mut_receive_at(other, Utc::now());
    }

    /// Like `mut_receive`, but gives you the a HLC instead of mutating.
    ///
    /// TODO: might be better to be private.
    #[must_use]
    pub fn receive_at(&self, other: &Self, now: DateTime<Utc>) -> Self {
        let mut next = *self;
        next.mut_receive_at(other, now);
        next
    }

    /// Like `mut_receive`, but gives you the a HLC instead of mutating.
    #[must_use]
    pub fn receive(&self, other: &Self) -> Self {
        self.receive_at(other, Utc::now())
    }

    /// Get the timestamp of this HLC.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the counter of this HLC.
    pub fn counter(&self) -> u16 {
        self.counter
    }

    /// Get the node ID of this HLC.
    pub fn node(&self) -> NodeId {
        self.node
    }
}

impl Ord for Hlc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.counter.cmp(&other.counter))
            .then(self.node.cmp(&other.node))
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}::{}", self.timestamp, self.counter, self.node)
    }
}

// impl FromRow for Hlc {
//     fn from_row<R: sqlx::Row>(row: &R) -> Result<Self, sqlx::Error> {
//         Ok(Self {
//             timestamp: row.try_get("timestamp")?,
//             counter: row.try_get("counter")?,
//             node: row.try_get("node")?,
//         })
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;
    use proptest::{prop_assert, prop_assume, proptest};

    proptest! {
        #[test]
        fn zero_is_less_than_every_other_hlc(other: Hlc) {
            let zero = Hlc::zero();

            prop_assume!(zero != other);
            prop_assert!(zero < other);
        }
    }

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
