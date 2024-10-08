use crate::op::Op;
use chrono::{DateTime, Utc};
use color_eyre::Result;
use rand_core::RngCore;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pings: HashMap<DateTime<Utc>, Ping>,
    lambda: f64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            pings: HashMap::new(),
            lambda: 45.0 / 60.0,
        }
    }
}

impl Document {
    pub fn fill(&mut self) -> Result<()> {
        // if self.pings.is_empty() {
        //     self.pings.push(Ping::default());
        // }

        // let now = Utc::now();
        // let mut current = self
        //     .pings
        //     .last()
        //     .expect("there to be at least one ping after backfilling");

        // while current.time <= now {
        //     let mut gen = Pcg32::new(current.time.timestamp().try_into()?, 0xa02bdbf7bb3c0a7);
        //     let adjustment = (gen.next_u32() as f64 / u32::MAX as f64).ln() / self.lambda * -1.0;
        //     let delta = chrono::Duration::minutes((adjustment * 60.0).floor() as i64);

        //     let next = Ping {
        //         time: current.time + delta,
        //         tag: None,
        //     };
        //     self.pings.push(next);
        //     current = self
        //         .pings
        //         .last()
        //         .expect("there to be a last ping after pushing");
        // }
        Ok(())
    }

    pub fn current_mut(&mut self) -> Option<&mut Ping> {
        // let now = Utc::now();

        // self.pings.iter_mut().rev().find(|p| p.time <= now)

        None
    }

    pub fn future(&self) -> Option<&Ping> {
        // let now = Utc::now();

        // self.pings.iter().rev().find(|p| p.time > now)
        None
    }

    pub fn apply_op(&mut self, op: &Op) {
        match op {
            Op::AddPing { when } => {
                self.add_ping(when);
            }

            Op::SetTag { when, tag } => {
                let ping = self.add_ping(when);
                ping.tag = Some(tag.clone())
            }

            Op::SetLambda { lambda } => {
                self.lambda = *lambda;
            }

            _ => todo!(),
        }
    }

    fn add_ping(&mut self, when: &DateTime<Utc>) -> &mut Ping {
        self.pings.entry(*when).or_insert(Ping {
            time: *when,
            tag: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub time: DateTime<Utc>,
    pub tag: Option<String>,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            tag: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod apply_op {
        use super::*;

        #[test]
        fn add_ping() {
            let mut doc = Document::default();
            let op = Op::AddPing { when: Utc::now() };

            doc.apply_op(&op);

            assert_eq!(doc.pings.len(), 1);
        }

        #[test]
        fn add_ping_idempotent() {
            let mut doc = Document::default();
            let op = Op::AddPing { when: Utc::now() };

            doc.apply_op(&op);
            doc.apply_op(&op);

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

            doc.apply_op(&op);

            assert_eq!(
                doc.pings.get(&when).and_then(|p| p.tag.clone()),
                Some("test".into())
            )
        }

        #[test]
        fn set_lambda() {
            let mut doc = Document::default();
            let op = Op::SetLambda { lambda: 1.0 };

            doc.apply_op(&op);

            assert_eq!(doc.lambda, 1.0);
        }
    }
}
