use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(PartialEq, Eq, Clone)]
pub struct Hlc {
    timestamp: DateTime<Utc>,
    counter: u64,
    node: Uuid,
}

impl Hlc {
    pub fn new(node: Uuid) -> Self {
        Self {
            timestamp: Utc::now(),
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

    pub fn next(&self) -> Self {
        let mut next = self.clone();
        next.increment();
        next
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
                node: Uuid::new_v4(),
            };
            let hlc2 = Hlc {
                timestamp: now + Duration::seconds(1),
                counter: 1,
                node: Uuid::new_v4(),
            };
            assert!(hlc1 < hlc2);
        }

        #[test]
        fn counter_is_considered_second() {
            let now = Utc::now();

            let hlc1 = Hlc {
                timestamp: now,
                counter: 0,
                node: Uuid::new_v4(),
            };
            let hlc2 = Hlc {
                timestamp: now,
                counter: 1,
                node: Uuid::new_v4(),
            };
            assert!(hlc1 < hlc2);
        }

        #[test]
        fn node_is_considered_third() {
            let now = Utc::now();

            let hlc1 = Hlc {
                timestamp: now,
                counter: 0,
                node: Uuid::nil(),
            };
            let hlc2 = Hlc {
                timestamp: now,
                counter: 0,
                node: Uuid::max(),
            };
            assert!(hlc1 < hlc2);
        }
    }

    mod next {
        use super::*;

        #[test]
        fn increments_counter_when_timestamp_is_in_the_past() {
            let now = Utc::now();
            let node = Uuid::new_v4();

            let mut hlc = Hlc {
                timestamp: now + Duration::seconds(1),
                counter: 0,
                node,
            };
            hlc.increment_at(now);

            // changed
            assert_eq!(hlc.counter, 1);

            // unchanged
            assert_eq!(hlc.timestamp, now + Duration::seconds(1));
            assert_eq!(hlc.node, node);
        }

        #[test]
        fn increments_timstamp_when_timestamp_is_in_the_future() {
            let now = Utc::now();
            let node = Uuid::new_v4();

            let mut hlc = Hlc {
                timestamp: now - Duration::seconds(1),
                counter: 1,
                node,
            };
            hlc.increment_at(now);

            // changed
            assert_eq!(hlc.timestamp, now);
            assert_eq!(hlc.counter, 0);

            // unchanged
            assert_eq!(hlc.node, node);
        }
    }
}
