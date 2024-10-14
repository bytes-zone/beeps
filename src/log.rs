use crate::hlc::Hlc;
use crate::op::Op;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimestampedOp {
    pub timestamp: Hlc,
    pub op: Op,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    ops: Vec<TimestampedOp>,
}

impl Log {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn push_unchecked(&mut self, op: TimestampedOp) {
        self.ops.push(op);
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
}
