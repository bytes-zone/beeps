use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::op::Op;
use chrono::{DateTime, Utc};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    ops: Vec<TimestampedOp>,
    clock: Hlc,
    pings: HashMap<DateTime<Utc>, Ping>,
    lambda: Lww<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimestampedOp {
    pub timestamp: Hlc,
    pub op: Op,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            ops: Vec::new(),
            clock: Hlc::new(0), // TODO: is this really the best default node ID?
            pings: HashMap::new(),
            lambda: Lww::new(45.0 / 60.0),
        }
    }
}

impl Document {
    pub fn from_ops(ops: &[TimestampedOp]) -> Self {
        let mut doc = Self::default();

        for op in ops {
            doc.apply_op(op);
        }

        doc
    }

    fn next_clock(&mut self) -> &Hlc {
        self.clock = self.clock.next();

        &self.clock
    }

    pub fn fill(&mut self) {
        let now = Utc::now();

        if self.pings.is_empty() {
            self.add_ping(&now);

            let next_clock = self.next_clock().clone();
            self.ops.push(TimestampedOp {
                timestamp: next_clock,
                op: Op::AddPing { when: now },
            });
        }

        let mut current = match self.current() {
            Some(ping) => ping.time,

            // If we have no current ping, even after backfilling, then all the
            // pings must be in the future and we don't need to do anything.
            None => return,
        };

        while current <= now {
            let mut gen = Pcg32::new(current.timestamp() as u64, 0xa02bdbf7bb3c0a7);
            let adjustment = (gen.next_u32() as f64 / u32::MAX as f64).ln() / *self.lambda * -1.0;
            let delta = chrono::Duration::minutes((adjustment * 60.0).floor() as i64);

            let next = current + delta;

            self.add_ping(&next);

            let next_clock = self.next_clock().clone();
            self.ops.push(TimestampedOp {
                timestamp: next_clock,
                op: Op::AddPing { when: now },
            });

            current = next
        }
    }

    pub fn current(&mut self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings
            .iter()
            .filter(|(_, v)| v.time <= now)
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
    }

    pub fn future(&self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings
            .iter()
            .filter(|(_, v)| v.time > now)
            .max_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
    }

    pub fn apply_op(&mut self, op: &TimestampedOp) {
        match &op.op {
            Op::AddPing { when } => {
                self.add_ping(when);
            }

            Op::SetTag { when, tag } => {
                let ping = self.add_ping(when);
                ping.tag.update(&op.timestamp, Some(tag.clone()));
            }

            Op::SetLambda { lambda } => self.lambda.update(&op.timestamp, *lambda),
        }

        self.ops.push(op.clone())
    }

    fn add_ping(&mut self, when: &DateTime<Utc>) -> &mut Ping {
        self.pings.entry(*when).or_insert(Ping {
            time: *when,
            tag: Lww::new(None),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    mod fill {
        use super::*;

        #[test]
        fn fills_empty_document() {
            let mut doc = Document::default();

            doc.fill();

            assert_eq!(doc.pings.len(), 2);
        }

        #[test]
        fn fills_document_with_future_ping() {
            let mut doc = Document::default();
            doc.add_ping(&(Utc::now() + chrono::Duration::hours(1)));

            doc.fill();

            assert_eq!(doc.pings.len(), 1);
        }

        #[test]
        fn fills_document_with_past_ping() {
            let mut doc = Document::default();
            doc.add_ping(&(Utc::now() - chrono::Duration::hours(1)));

            doc.fill();

            assert!(!doc.ops.is_empty());
        }
    }

    mod apply_op {
        use super::*;

        #[test]
        fn add_ping() {
            let mut doc = Document::default();
            let op = Op::AddPing { when: Utc::now() };

            doc.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(doc.pings.len(), 1);
        }

        #[test]
        fn add_ping_idempotent() {
            let mut doc = Document::default();
            let op = Op::AddPing { when: Utc::now() };
            let clock = Hlc::new(0);

            doc.apply_op(&TimestampedOp {
                timestamp: clock.clone(),
                op: op.clone(),
            });
            doc.apply_op(&TimestampedOp {
                timestamp: clock.clone(),
                op: op.clone(),
            });

            assert_eq!(doc.pings.len(), 1);
        }

        #[test]
        fn set_tag() {
            let mut doc = Document::default();
            let when = Utc::now();
            let op = Op::SetTag {
                when,
                tag: "test".into(),
            };

            doc.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(
                doc.pings.get(&when).and_then(|p| p.tag.clone()),
                Some("test".into())
            )
        }

        #[test]
        fn set_lambda() {
            let mut doc = Document::default();
            let op = Op::SetLambda { lambda: 1.0 };

            doc.apply_op(&TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(*doc.lambda, 1.0);
        }
    }
}
