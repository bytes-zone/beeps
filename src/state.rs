use crate::log::TimestampedOp;
use crate::lww::Lww;
use crate::op::Op;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct State {
    pub pings: HashMap<DateTime<Utc>, Ping>,
}

impl State {
    #[tracing::instrument(skip(self), level = "trace")]
    pub fn apply_op(&mut self, op: &TimestampedOp) {
        match &op.op {
            Op::AddPing { when } => {
                self.add_ping(when);
            }

            Op::SetTag { when, tag } => {
                let ping = self.add_ping(when);
                ping.tag.update(&op.timestamp, Some(tag.clone()));
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_ping(&mut self, when: &DateTime<Utc>) -> &mut Ping {
        tracing::debug!("adding ping with no tag");

        self.pings.entry(*when).or_insert(Ping {
            time: *when,
            tag: Lww::new(None),
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn latest(&self) -> Option<&Ping> {
        self.pings.iter().max_by_key(|(k, _)| *k).map(|(_, v)| v)
    }

    #[tracing::instrument(skip(self))]
    pub fn current(&self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings
            .iter()
            .filter(|(_, v)| v.time <= now)
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
    }

    #[tracing::instrument(skip(self))]
    pub fn future(&self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings
            .iter()
            .filter(|(_, v)| v.time > now)
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_ping(&self, when: &DateTime<Utc>) -> Option<&Ping> {
        self.pings.get(when)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ping {
    pub time: DateTime<Utc>,
    pub tag: Lww<Option<String>>,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            tag: Lww::new(None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hlc::Hlc;

    mod apply_op {
        use super::*;

        #[test]
        fn add_ping() {
            let mut state = State::default();
            let op = Op::AddPing { when: Utc::now() };

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(state.pings.len(), 1);
        }

        #[test]
        fn add_ping_idempotent() {
            let mut state = State::default();
            let op = Op::AddPing { when: Utc::now() };
            let clock = Hlc::new(0);

            state.apply_op(&TimestampedOp {
                timestamp: clock.clone(),
                op: op.clone(),
            });
            state.apply_op(&TimestampedOp {
                timestamp: clock.clone(),
                op: op.clone(),
            });

            assert_eq!(state.pings.len(), 1);
        }

        #[test]
        fn set_tag() {
            let mut state = State::default();
            let when = Utc::now();
            let op = Op::SetTag {
                when,
                tag: "test".into(),
            };

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(
                state.pings.get(&when).and_then(|p| p.tag.clone()),
                Some("test".into())
            )
        }
    }

    mod latest {
        use super::*;

        #[test]
        fn returns_latest_ping() {
            let mut state = State::default();
            let now = Utc::now();
            let later = now + chrono::Duration::seconds(1);

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: now },
            });

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: later },
            });

            assert_eq!(state.latest().map(|p| p.time), Some(later));
        }
    }

    mod current {
        use super::*;

        #[test]
        fn returns_current_ping() {
            let mut state = State::default();
            let now = Utc::now();
            let later = now + chrono::Duration::seconds(1);

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: now },
            });

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: later },
            });

            assert_eq!(state.current().map(|p| p.time), Some(now));
        }

        #[test]
        fn returns_nothing_if_all_pings_are_in_future() {
            let mut state = State::default();
            let later = Utc::now() + chrono::Duration::seconds(1);

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: later },
            });

            assert_eq!(state.current(), None);
        }
    }

    mod future {
        use super::*;

        #[test]
        fn returns_future_ping() {
            let mut state = State::default();
            let now = Utc::now();
            let later = now + chrono::Duration::seconds(1);

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: now },
            });

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: later },
            });

            assert_eq!(state.future().map(|p| p.time), Some(later));
        }

        #[test]
        fn returns_nothing_if_all_pings_are_in_past() {
            let mut state = State::default();
            let now = Utc::now();

            state.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op: Op::AddPing { when: now },
            });

            assert_eq!(state.future(), None);
        }
    }
}
