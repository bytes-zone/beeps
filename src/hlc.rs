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

    #[cfg(test)]
    pub fn new_at(node: u8, timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            counter: 0,
            node,
        }
    }

    pub fn next(&self, node: u8) -> Self {
        let now = Utc::now();

        if now > self.timestamp {
            Self {
                timestamp: now,
                counter: 0,
                node,
            }
        } else {
            Self {
                timestamp: self.timestamp,
                counter: self.counter + 1,
                node,
            }
        }
    }

    pub fn next_tiebreak(&self, other: Option<&Self>, node: u8) -> Self {
        let current = match other {
            Some(other_) => self.max(other_),
            None => self,
        };

        current.next(node)
    }
}

impl PartialOrd for Hlc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hlc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.timestamp != other.timestamp {
            self.timestamp.cmp(&other.timestamp)
        } else if self.counter != other.counter {
            return self.counter.cmp(&other.counter);
        } else {
            return self.node.cmp(&other.node);
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

            let next = hlc.next(hlc.node);

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

            let next = hlc.next(hlc.node);

            assert!(next.timestamp > hlc.timestamp);
            assert_eq!(next.counter, 0);
        }
    }

    mod next_tiebreak {
        use super::*;
        use chrono::TimeZone;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn clock_monotonically_increases(node_a in 0u8..=255, counter_a in 0u64.., ts_a in 0i64..=2000000000, node_b in 0u8..=255, counter_b in 0u64.., ts_b in 0i64..=2000000000) {
                let hlc_a = Hlc {
                    timestamp: Utc.timestamp_opt(ts_a, 0).unwrap(),
                    counter: counter_a,
                    node: node_a,
                };

                let hlc_b = Hlc {
                    timestamp: Utc.timestamp_opt(ts_b, 0).unwrap(),
                    counter: counter_b,
                    node: node_b,
                };

                let next_hlc = hlc_a.next_tiebreak(Some(&hlc_b), node_a);

                assert!(next_hlc >= hlc_a, "{next_hlc:?} < {hlc_a:?}");
                assert!(next_hlc >= hlc_b, "{next_hlc:?} < {hlc_b:?}");
            }
        }
    }

    mod ord {
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

            assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
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

            assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
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

            assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
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

            assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
        }
    }
}
