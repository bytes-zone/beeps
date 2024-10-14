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

    pub fn latest(&self) -> Option<&Ping> {
        self.pings.iter().max_by_key(|(k, _)| *k).map(|(_, v)| v)
    }

    pub fn current(&self) -> Option<&Ping> {
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

    pub fn get_ping(&self, when: &DateTime<Utc>) -> Option<&Ping> {
        self.pings.get(when)
    }
}

#[derive(Debug)]
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
