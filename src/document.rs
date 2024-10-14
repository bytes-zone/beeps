use crate::hlc::Hlc;
use crate::log::{Log, TimestampedOp};
use crate::lww::Lww;
use crate::op::Op;
use crate::state::{Ping, State};
use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{OptionExt, WrapErr},
    Result,
};
use rand_core::RngCore;
use rand_pcg::Pcg32;

#[derive(Debug)]
pub struct Document {
    log: Log,
    clock: Hlc,
    lambda: Lww<f64>,
    state: State,
}

impl Document {
    #[tracing::instrument(skip(log), level = "trace")]
    pub fn from_ops(log: Log) -> Self {
        let mut state = State::default();

        for op in log.ops() {
            state.apply_op(op);
        }

        Self {
            log,
            clock: Hlc::new(0), // TODO: allow setting a node ID, maybe from log?
            lambda: Lww::new(1.0 / 45.0),
            state,
        }
    }

    pub fn empty() -> Self {
        Self::from_ops(Log::default())
    }

    #[tracing::instrument(skip(self, wall_clock))]
    pub fn fill(&mut self, wall_clock: impl WallClock) -> Result<()> {
        let now = wall_clock.now();

        if self.state.pings.is_empty() {
            tracing::debug!(when = ?now, "pings is empty, adding initial ping");
            self.add_ping(&now).wrap_err("could not add ping")?;
        }

        let mut current = match self.latest() {
            Some(ping) => {
                tracing::debug!(?ping.time, future = ping.time > now, "had a latest ping");
                ping.time
            }

            // If we have no current ping, even after backfilling, then all the
            // pings must be in the future and we don't need to do anything.
            None => return Ok(()),
        };

        while current <= now {
            let next = self.next_time(current);
            tracing::debug!(?next, "got next time");

            self.add_ping(&next).wrap_err("could not add ping")?;

            current = next
        }

        Ok(())
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
        self.state.latest()
    }

    pub fn current(&self) -> Option<&Ping> {
        self.state.current()
    }

    pub fn future(&self) -> Option<&Ping> {
        self.state.future()
    }

    #[tracing::instrument(skip(self))]
    fn add_ping(&mut self, when: &DateTime<Utc>) -> Result<()> {
        tracing::debug!("adding ping with no tag");

        self.clock = self.clock.next(self.clock.node);

        let op = TimestampedOp {
            timestamp: self.clock.clone(),
            op: Op::AddPing { when: *when },
        };

        self.state.apply_op(&op);
        self.log.push(op).wrap_err("could not push operation")?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_tag(&mut self, when: &DateTime<Utc>, tag: String) -> Result<()> {
        tracing::debug!("setting tag"); // arguments are added by tracing::instrument

        let ping = self
            .state
            .get_ping(when)
            .ok_or_eyre("provided ping does not exist")?;

        self.clock = self
            .clock
            .next_tiebreak(ping.tag.timestamp(), self.clock.node);

        let op = TimestampedOp {
            timestamp: self.clock.clone(),
            op: Op::SetTag { when: *when, tag },
        };

        self.state.apply_op(&op);
        self.log.push(op).wrap_err("could not push operation")?;

        Ok(())
    }

    pub fn log(&self) -> &Log {
        &self.log
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
            let mut doc = Document::empty();

            doc.fill(Utc).unwrap();

            assert_eq!(doc.state.pings.len(), 2);
        }

        #[test]
        fn fills_document_with_future_ping() {
            let mut doc = Document::empty();

            let clock = FixedClock::freeze();

            doc.add_ping(&(clock.now() + chrono::Duration::hours(1)))
                .unwrap();

            doc.fill(clock).unwrap();

            assert_eq!(doc.state.pings.len(), 1);
        }

        #[test]
        fn fills_document_with_past_ping() {
            let mut doc = Document::empty();
            let clock = FixedClock::freeze();
            doc.add_ping(&(clock.now() - chrono::Duration::hours(1)))
                .unwrap();

            doc.fill(clock).unwrap();

            assert!(!doc.log.is_empty());
        }

        #[test]
        fn does_not_add_pings_if_we_have_a_future_ping() {
            let mut doc = Document::empty();
            doc.fill(Utc).unwrap();

            // Now that we've filled pings, we should have a future ping. Just to check...
            assert!(doc.future().is_some());

            // A subsequent call to fill should not add any more operations
            let num_ops = doc.log.len();
            doc.fill(Utc).unwrap();

            assert_eq!(num_ops, doc.log.len());
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

                let doc = Document::empty();
                let next = doc.next_time(start);

                assert!(next >= start + Duration::seconds(1), "{next:#?} was not GE than {start:#?}")
            }

            #[test]
            fn averages_to_lambda(lambda_minutes in 1u32..120) {
                let mut doc = Document::empty();
                doc.lambda.update(&doc.clock, 1.0 / lambda_minutes as f64);

                let mut current = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
                let mut deltas = Vec::new();

                for _ in 0..2500 {
                    let next = doc.next_time(current);
                    deltas.push(next - current);
                    current = next
                }

                let average = deltas.iter().sum::<chrono::Duration>().num_minutes() as f64 / deltas.len() as f64;

                let diff = (average / lambda_minutes as f64).abs() - 1.0;
                assert!(diff < 0.05, "{:0.2} was not close to {:0.2} ({:0.4})", average, lambda_minutes, diff);
            }
        }
    }

    mod add_ping {
        use super::*;

        #[test]
        fn adds_ping() {
            let mut doc = Document::empty();
            let now = Utc::now();
            doc.add_ping(&now).unwrap();

            assert_eq!(doc.state.pings.len(), 1);
            assert_eq!(*doc.state.pings[&now].tag, None);
        }

        #[test]
        fn advances_clock() {
            let mut doc = Document::empty();

            let orig_clock = doc.clock.clone();

            let ping_time = Utc::now();
            doc.add_ping(&ping_time).unwrap();

            assert!(doc.clock > orig_clock, "{:?} <= {orig_clock:?}", doc.clock);
        }
    }

    mod set_tag {
        use super::*;

        #[test]
        fn sets_tag() {
            let mut doc = Document::empty();
            let now = Utc::now();
            doc.add_ping(&now).unwrap();

            doc.set_tag(&now, "test".to_string()).unwrap();

            assert_eq!(*doc.state.pings[&now].tag, Some("test".to_string()));
        }

        #[test]
        fn sets_tag_error() {
            let mut doc = Document::empty();
            let now = Utc::now();
            // no add_ping here!

            let result = doc.set_tag(&now, "test".to_string());

            assert!(result.is_err());
        }

        #[test]
        fn advances_clock() {
            let mut doc = Document::empty();

            let ping_time = Utc::now();
            doc.add_ping(&ping_time).unwrap();

            let orig_clock = doc.clock.clone();

            doc.set_tag(&ping_time, "test".to_string()).unwrap();

            assert!(doc.clock > orig_clock, "{:?} <= {orig_clock:?}", doc.clock);
        }
    }
}
