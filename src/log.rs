use std::collections::HashMap;

use crate::hlc::Hlc;
use crate::op::Op;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimestampedOp {
    pub timestamp: Hlc,
    pub op: Op,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("new operation was before last existing operation")]
    OrderingViolation,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Log {
    ops: Vec<TimestampedOp>,
}

impl Log {
    pub fn from_ops(ops: Vec<TimestampedOp>) -> Self {
        Self { ops }
    }

    #[deprecated(note = "use from_ops and then checked pushes")]
    pub fn push_unchecked(&mut self, op: TimestampedOp) {
        self.ops.push(op);
    }

    pub fn push(&mut self, op: TimestampedOp) -> Result<(), Error> {
        if let Some(last_op) = self.latest_for_node(op.timestamp.node) {
            if last_op.timestamp > op.timestamp {
                return Err(Error::OrderingViolation);
            }
        }

        self.ops.push(op);

        Ok(())
    }

    fn latest_for_node(&self, node: u8) -> Option<&TimestampedOp> {
        self.ops.iter().rev().find(|op| op.timestamp.node == node)
    }

    pub fn ops(&self) -> &Vec<TimestampedOp> {
        &self.ops
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[tracing::instrument(skip(self))]
    pub fn latest_for_each_node(&self) -> HashMap<u8, Hlc> {
        let mut latest_ops: HashMap<u8, Hlc> = HashMap::with_capacity(8);

        for op in &self.ops {
            let node = op.timestamp.node;

            let latest = latest_ops
                .entry(node)
                .or_insert_with(|| op.timestamp.clone());
            if *latest < op.timestamp {
                *latest = op.timestamp.clone();
            }
        }

        latest_ops
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{Duration, Utc};

    mod push {

        use super::*;

        #[test]
        fn pushes_first_op() {
            let mut log = Log::default();

            let op = TimestampedOp {
                timestamp: Hlc::new(1),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            assert!(log.push(op).is_ok());
            assert_eq!(log.ops.len(), 1);
        }

        #[test]
        fn rejects_out_of_order_pushes() {
            let mut log = Log::default();

            let ts1 = Utc::now();
            let op1 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts1),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            let op2 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts1 - Duration::seconds(1)),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            assert!(log.push(op1).is_ok());
            assert_eq!(log.push(op2).unwrap_err(), Error::OrderingViolation);
        }

        #[test]
        fn out_of_order_pushes_are_ok_if_they_have_different_node_ids() {
            let mut log = Log::default();

            let ts1 = Utc::now();
            let op1 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts1),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            let op2 = TimestampedOp {
                timestamp: Hlc::new_at(2, ts1 - Duration::seconds(1)),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            assert!(log.push(op1).is_ok());
            assert!(log.push(op2).is_ok());
        }
    }

    mod latest_for_each_node {
        use super::*;

        #[test]
        fn returns_latest_op_for_single_node() {
            let mut log = Log::default();

            let ts1 = Utc::now();
            let ts2 = ts1 + Duration::seconds(1);

            let op1 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts1),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            let op2 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts2),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            log.push(op1).unwrap();
            log.push(op2).unwrap();

            let latest = log.latest_for_each_node();

            assert_eq!(latest.get(&1).unwrap(), &Hlc::new_at(1, ts2));
        }

        #[test]
        fn returns_latest_op_for_multiple_nodes() {
            let mut log = Log::default();

            let ts1 = Utc::now();
            let ts2 = ts1 + Duration::seconds(1);
            let ts3 = ts1 + Duration::seconds(1);

            let op1 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts1),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            let op2 = TimestampedOp {
                timestamp: Hlc::new_at(2, ts2),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            let op3 = TimestampedOp {
                timestamp: Hlc::new_at(1, ts3),
                op: Op::SetTag {
                    when: Utc::now(),
                    tag: "tag".to_string(),
                },
            };

            log.push(op1).unwrap();
            log.push(op2).unwrap();
            log.push(op3).unwrap();

            let latest = log.latest_for_each_node();

            assert_eq!(latest.get(&1).unwrap(), &Hlc::new_at(1, ts3));
            assert_eq!(latest.get(&2).unwrap(), &Hlc::new_at(2, ts2));
        }
    }
}
