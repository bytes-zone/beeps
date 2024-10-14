use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::op::Op;
use chrono::{DateTime, Utc};
use color_eyre::{eyre::OptionExt, Result};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
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
            lambda: Lww::new(1.0 / 45.0),
        }
    }
}

impl Document {
    #[tracing::instrument(skip(ops), level = "trace")]
    pub fn from_ops(ops: Vec<TimestampedOp>) -> Self {
        let mut doc = Self::default();

        for op in ops {
            doc.apply_op(op);
        }

        doc
    }

    fn next_clock(&mut self) -> Hlc {
        self.clock.next(self.clock.node)
    }

    #[tracing::instrument(skip(self, wall_clock))]
    pub fn fill(&mut self, wall_clock: impl WallClock) {
        let now = wall_clock.now();

        if self.pings.is_empty() {
            tracing::debug!(when = ?now, "pings is empty, adding initial ping");
            self.add_ping(&now);
        }

        let mut current = match self.latest() {
            Some(ping) => {
                tracing::debug!(?ping.time, future = ping.time > now, "had a latest ping");
                ping.time
            }

            // If we have no current ping, even after backfilling, then all the
            // pings must be in the future and we don't need to do anything.
            None => return,
        };

        while current <= now {
            let next = self.next_time(current);
            tracing::debug!(?next, "got next time");

            self.add_ping(&next);

            current = next
        }
    }

    fn next_time(&self, current: DateTime<Utc>) -> DateTime<Utc> {
        let mut gen = Pcg32::new(current.timestamp() as u64, 0xa02bdbf7bb3c0a7);
        let base = gen.next_u32() as f64 / u32::MAX as f64;

        // We expect the delta to be `1 / minutes`, but we get fractional
        // minutes from this calculation. We multiply by 60 to change to
        // seconds, then round up to the next second to calculate the delta.
        let adjustment = base.ln() / *self.lambda * -60.0;
        let delta = chrono::Duration::seconds(adjustment.ceil() as i64);

        current + delta
    }

    fn latest(&self) -> Option<&Ping> {
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

    #[tracing::instrument(skip(self))]
    pub fn set_tag(&mut self, when: &DateTime<Utc>, tag: String) -> Result<()> {
        tracing::debug!("setting tag"); // arguments are added by tracing::instrument

        let ping = self
            .pings
            .get_mut(when)
            .ok_or_eyre("provided ping does not exist")?;

        let timestamp = self
            .clock
            .next_tiebreak(ping.tag.timestamp(), self.clock.node);

        self.clock = timestamp;

        ping.tag.update(&self.clock, Some(tag.clone()));

        self.ops.push(TimestampedOp {
            timestamp: self.clock.clone(),
            op: Op::SetTag { when: *when, tag },
        });

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "trace")]
    pub fn apply_op(&mut self, op: TimestampedOp) {
        match &op.op {
            Op::AddPing { when } => {
                self.pings.entry(*when).or_insert_with(|| Ping {
                    time: *when,
                    tag: Lww::new(None),
                });
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

        self.clock = self.next_clock();

        self.ops.push(TimestampedOp {
            timestamp: self.clock.clone(),
            op: Op::AddPing { when: *when },
        });

        self.pings.entry(*when).or_insert(Ping {
            time: *when,
            tag: Lww::new(None),
        })
    }

    pub fn ops(&self) -> &Vec<TimestampedOp> {
        &self.ops
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

trait WallClock {
    fn now(&self) -> DateTime<Utc>;
}

impl WallClock for Utc {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
struct FixedClock {
    time: DateTime<Utc>,
}

#[cfg(test)]
impl FixedClock {
    fn freeze() -> Self {
        Self { time: Utc::now() }
    }
}

#[cfg(test)]
impl WallClock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.time
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

            doc.fill(Utc);

            assert_eq!(doc.pings.len(), 2);
        }

        #[test]
        fn fills_document_with_future_ping() {
            let mut doc = Document::default();

            let clock = FixedClock::freeze();

            doc.add_ping(&(clock.now() + chrono::Duration::hours(1)));

            doc.fill(clock);

            assert_eq!(doc.pings.len(), 1);
        }

        #[test]
        fn fills_document_with_past_ping() {
            let mut doc = Document::default();
            let clock = FixedClock::freeze();
            doc.add_ping(&(clock.now() - chrono::Duration::hours(1)));

            doc.fill(clock);

            assert!(!doc.ops.is_empty());
        }

        #[test]
        fn does_not_add_pings_if_we_have_a_future_ping() {
            let mut doc = Document::default();
            doc.fill(Utc);

            // Now that we've filled pings, we should have a future ping. Just to check...
            assert!(doc.future().is_some());

            // A subsequent call to fill should not add any more operations
            let num_ops = doc.ops.len();
            doc.fill(Utc);

            assert_eq!(num_ops, doc.ops.len());
        }
    }

    mod next_time {
        use super::*;
        use chrono::{Duration, TimeZone};
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn always_advances_by_at_least_a_minute(minutes in 0i64..1_000_000) {
                let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap() + Duration::minutes(minutes);

                let doc = Document::default();
                let next = doc.next_time(start);

                assert!(next >= start + Duration::seconds(1), "{next:#?} was not GE than {start:#?}")
            }

            #[test]
            fn averages_to_lambda(lambda_minutes in 1u32..120) {
                let mut doc = Document::default();
                doc.lambda.update(&doc.clock, 1.0 / lambda_minutes as f64);

                let mut current = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
                let mut deltas = Vec::new();

                for _ in 0..2500 {
                    let next = doc.next_time(current);
                    deltas.push(next - current);
                    current = next
                }

                let average = deltas.iter().sum::<chrono::Duration>().num_minutes() as f64 / deltas.len() as f64;

                // let diff = (average - lambda_minutes).abs();
                let diff = (average / lambda_minutes as f64).abs() - 1.0;
                assert!(diff < 0.05, "{:0.2} was not close to {:0.2} ({:0.4})", average, lambda_minutes, diff);
            }
        }
    }

    mod set_tag {
        use super::*;

        #[test]
        fn sets_tag() {
            let mut doc = Document::default();
            let now = Utc::now();
            doc.add_ping(&now);

            doc.set_tag(&now, "test".to_string()).unwrap();

            assert_eq!(*doc.pings[&now].tag, Some("test".to_string()));
        }

        #[test]
        fn sets_tag_error() {
            let mut doc = Document::default();
            let now = Utc::now();
            // no add_ping here!

            let result = doc.set_tag(&now, "test".to_string());

            assert!(result.is_err());
        }

        #[test]
        fn sets_clock() {
            let mut doc = Document::default();

            let ping_time = Utc::now();
            doc.add_ping(&ping_time);

            let orig_clock = doc.clock.clone();

            doc.set_tag(&ping_time, "test".to_string()).unwrap();

            assert!(doc.clock > orig_clock, "{:?} <= {orig_clock:?}", doc.clock);
        }
    }

    mod apply_op {
        use super::*;

        #[test]
        fn add_ping() {
            let mut doc = Document::default();
            let op = Op::AddPing { when: Utc::now() };

            doc.apply_op(TimestampedOp {
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

            doc.apply_op(TimestampedOp {
                timestamp: clock.clone(),
                op: op.clone(),
            });
            doc.apply_op(TimestampedOp {
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

            doc.apply_op(TimestampedOp {
                timestamp: Hlc::new(0),
                op,
            });

            assert_eq!(
                doc.pings.get(&when).and_then(|p| p.tag.clone()),
                Some("test".into())
            )
        }
    }
}
