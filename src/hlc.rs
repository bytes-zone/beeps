use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hlc {
    pub timestamp: DateTime<Utc>,
    pub counter: u64,
    pub node: u8,
}

impl Hlc {
    pub fn new(node: u8) -> Self {
        Self {
            timestamp: Utc::now(),
            counter: 0,
            node,
        }
    }

    pub fn next(&self) -> Self {
        let now = Utc::now();

        if now > self.timestamp {
            Self {
                timestamp: now,
                counter: 0,
                node: self.node,
            }
        } else {
            Self {
                timestamp: self.timestamp,
                counter: self.counter + 1,
                node: self.node,
            }
        }
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.timestamp != other.timestamp {
            self.timestamp.partial_cmp(&other.timestamp)
        } else if self.counter != other.counter {
            return self.counter.partial_cmp(&other.counter);
        } else {
            return self.node.partial_cmp(&other.node);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn creates_new_hlc() {
            let hlc = Hlc::new(0);

            assert_eq!(hlc.counter, 0);
            assert_eq!(hlc.node, 0);
        }
    }

    mod next {
        use super::*;

        #[test]
        fn bumps_counter_when_clock_slips_back() {
            let hlc = Hlc {
                timestamp: Utc::now() + chrono::Duration::seconds(1),
                counter: 0,
                node: 0,
            };

            let next = hlc.next();

            assert_eq!(next.timestamp, hlc.timestamp);
            assert_eq!(next.counter, 1);
        }

        #[test]
        fn bumps_timestamp_when_clock_goes_forward() {
            let hlc = Hlc {
                timestamp: Utc::now() - chrono::Duration::seconds(1),
                counter: 0,
                node: 0,
            };

            let next = hlc.next();

            assert!(next.timestamp > hlc.timestamp);
            assert_eq!(next.counter, 0);
        }
    }

    mod partial_ord {
        use super::*;

        #[test]
        fn equal() {
            let timestamp = Utc::now();

            let a = Hlc {
                timestamp,
                counter: 0,
                node: 0,
            };
            let b = Hlc {
                timestamp,
                counter: 0,
                node: 0,
            };

            assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));
        }

        #[test]
        fn differing_node_ids() {
            let timestamp = Utc::now();

            let a = Hlc {
                timestamp,
                counter: 0,
                node: 0,
            };
            let b = Hlc {
                timestamp,
                counter: 0,
                node: 1,
            };

            assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
        }

        #[test]
        fn differing_counters() {
            let timestamp = Utc::now();

            let a = Hlc {
                timestamp,
                counter: 0,
                node: 0,
            };
            let b = Hlc {
                timestamp,
                counter: 1,
                node: 0,
            };

            assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
        }

        #[test]
        fn differing_timestamps() {
            let a = Hlc {
                timestamp: Utc::now(),
                counter: 0,
                node: 0,
            };
            let b = Hlc {
                timestamp: Utc::now() + chrono::Duration::seconds(1),
                counter: 0,
                node: 0,
            };

            assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
        }
    }
}
