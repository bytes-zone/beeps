use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(PartialEq, Eq)]
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

    mod ord {
        use chrono::Duration;

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
}
