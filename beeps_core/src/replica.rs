use crate::hlc::Hlc;
use crate::node_id::NodeId;
use crate::scheduler::Scheduler;
use crate::{document::Document, merge::Merge};
use chrono::{DateTime, Utc};

/// The local state of a replica ("who am I" and "what do I know"). Reading the
/// state should be fairly straightforward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Replica {
    /// The clock used to write. Should always be higher than any clock in state.
    clock: Hlc,

    /// Data that this replica will write to and sync with peers.
    document: Document,
}

impl Replica {
    /// Create a new replica with the given node ID.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            clock: Hlc::new(node_id),
            document: Document::default(),
        }
    }

    /// Increment the clock and get the next value you should use. Always use
    /// this when writing to ensure that the replica-level clock is the highest
    /// in the state.
    #[must_use]
    fn next_clock(&mut self) -> Hlc {
        self.clock.increment();
        self.clock
    }

    /// Read the current state.
    pub fn state(&self) -> &Document {
        &self.document
    }

    /// Set the average number of minutes between pings.
    pub fn set_minutes_per_ping(&mut self, new: u16) {
        let clock = self.next_clock();
        self.document.set_minutes_per_ping(new, clock);
    }

    /// Add a ping, likely in coordination with a `Scheduler`.
    pub fn add_ping(&mut self, when: DateTime<Utc>) {
        self.document.add_ping(when);
    }

    /// Tag an existing ping (returns false if the ping cannot be tagged because
    /// it does not exist.)
    pub fn tag_ping(&mut self, when: DateTime<Utc>, tag: String) -> bool {
        let clock = self.next_clock();
        self.document.tag_ping(when, tag, clock)
    }

    /// Untag an existing ping (returns false if the ping cannot be tagged
    /// because it does not exist.)
    pub fn untag_ping(&mut self, when: DateTime<Utc>) -> bool {
        let clock = self.next_clock();
        self.document.untag_ping(when, clock)
    }

    /// Does the same as `schedule_ping` but allows you to specify the cutoff.
    /// Returns the list of pings that were scheduled.
    fn schedule_pings_with_cutoff(&mut self, cutoff: DateTime<Utc>) -> Vec<DateTime<Utc>> {
        let mut new_pings = Vec::new();

        let latest_ping = if let Some(ping) = self.document.latest_ping().copied() {
            ping
        } else {
            let now = Utc::now();
            self.document.pings.insert(now);
            new_pings.push(now);

            now
        };

        // Early check: if we already have a ping past the cutoff, we don't need
        // to do any more work.
        if latest_ping > cutoff {
            return new_pings;
        }

        let scheduler = Scheduler::new(*self.document.minutes_per_ping.value(), latest_ping);

        for next in scheduler {
            self.document.pings.insert(next);
            new_pings.push(next);

            // accepting one past the cutoff gets us into the future
            if next > cutoff {
                break;
            }
        }

        new_pings
    }

    /// Schedule pings into the future. We don't just schedule *up to* the given
    /// time, but go one past that. That means that if the given time is the
    /// current time, we end up with the time we should next notify at. Returns
    /// the list of pings that were scheduled.
    pub fn schedule_pings(&mut self) -> Vec<DateTime<Utc>> {
        self.schedule_pings_with_cutoff(Utc::now())
    }

    /// Get the current value of the given ping.
    pub fn get_tag(&self, ping: &DateTime<Utc>) -> Option<&String> {
        self.document.get_tag(ping)
    }

    /// Get all the pings that have been scheduled.
    pub fn pings(&self) -> impl DoubleEndedIterator<Item = &DateTime<Utc>> {
        self.document.pings.iter()
    }

    /// Get the document (for syncing)
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Merge another document into ours (for syncing)
    pub fn merge(&mut self, other: Document) {
        // TODO: make sure that our clock is higher than any clock in this document.
        self.document.merge_mut(other);
    }

    /// Replace our document with another (for initial syncs)
    pub fn replace_doc(&mut self, other: Document) {
        // TODO: make sure that our clock is higher than any clock in this document.
        self.document = other;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod schedule_pings {
        use super::*;

        #[test]
        fn fills_from_last_time_until_cutoff() {
            let mut doc = Replica::new(NodeId::random());

            let now = Utc::now();

            doc.set_minutes_per_ping(1);
            doc.add_ping(now - chrono::Duration::days(1));
            doc.schedule_pings();

            assert!(doc.state().pings.len() > 1);
        }

        #[test]
        fn fills_one_date_exactly_in_the_future() {
            let mut doc = Replica::new(NodeId::random());

            let now = Utc::now();

            doc.set_minutes_per_ping(1);
            doc.add_ping(now - chrono::Duration::days(1));
            doc.schedule_pings();

            assert_eq!(
                doc.state()
                    .pings
                    .iter()
                    .filter(|p| *p > &now)
                    .collect::<Vec<_>>()
                    .len(),
                1
            );
        }

        #[test]
        fn any_dates_filled_are_from_the_scheduler() {
            let mut doc = Replica::new(NodeId::random());

            let now = Utc::now();
            let start = now - chrono::Duration::days(1);

            doc.set_minutes_per_ping(1);
            doc.add_ping(start);
            doc.schedule_pings();

            let scheduler = Scheduler::new(1, start);
            let scheduled = scheduler.take(10).collect::<Vec<_>>();

            for date in scheduled {
                assert!(doc.state().pings.contains(&date));
            }
        }

        #[test]
        fn returns_true_if_any_pings_were_scheduled() {
            let mut doc = Replica::new(NodeId::random());

            let now = Utc::now();

            // Our first call is going to schedule some pings
            assert!(!doc.schedule_pings_with_cutoff(now).is_empty());

            // A second call won't schedule anything new, so we should not
            // schedule new pings
            assert_eq!(doc.schedule_pings_with_cutoff(now).len(), 0);
        }
    }

    mod state_machine {
        use super::*;
        use proptest::prelude::*;
        use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};

        #[derive(Debug, Clone)]
        enum Transition {
            SetMinutesPerPing(u16),
            AddPing(chrono::DateTime<Utc>),
            TagPing(chrono::DateTime<Utc>, String),
            UntagPing(chrono::DateTime<Utc>),
        }

        #[derive(Debug, Clone)]
        struct RefState {}

        impl ReferenceStateMachine for RefState {
            type State = RefState;

            type Transition = Transition;

            fn init_state() -> BoxedStrategy<Self::State> {
                Just(RefState {}).boxed()
            }

            fn transitions(_: &Self::State) -> BoxedStrategy<Self::Transition> {
                prop_oneof![
                    1 => (1..=4u16).prop_map(|i| Transition::SetMinutesPerPing(i * 15)),
                    10 => crate::test::timestamp_range(0..=2i64).prop_map(Transition::AddPing),
                    10 =>
                        (crate::test::timestamp_range(0..=2i64), "(a|b|c)")
                            .prop_map(|(ts, tag)| Transition::TagPing(ts, tag)),
                    5 =>
                        crate::test::timestamp_range(0..=2i64)
                            .prop_map(Transition::UntagPing),
                ]
                .boxed()
            }

            fn apply(state: Self::State, _: &Self::Transition) -> Self::State {
                state
            }
        }

        struct ReplicaStateMachine {}

        impl StateMachineTest for ReplicaStateMachine {
            type SystemUnderTest = Replica;

            type Reference = RefState;

            fn init_test(
                _: &<Self::Reference as proptest_state_machine::ReferenceStateMachine>::State,
            ) -> Self::SystemUnderTest {
                Replica::new(NodeId::random())
            }

            fn apply(
                mut state: Self::SystemUnderTest,
                _: &<Self::Reference as proptest_state_machine::ReferenceStateMachine>::State,
                transition: <Self::Reference as proptest_state_machine::ReferenceStateMachine>::Transition,
            ) -> Self::SystemUnderTest {
                match transition {
                    Transition::SetMinutesPerPing(new) => {
                        state.set_minutes_per_ping(new);
                    }
                    Transition::AddPing(when) => {
                        state.add_ping(when);
                    }
                    Transition::TagPing(when, tag) => {
                        state.tag_ping(when, tag.clone());
                    }
                    Transition::UntagPing(when) => {
                        state.untag_ping(when);
                    }
                }

                state
            }

            fn check_invariants(
                state: &Self::SystemUnderTest,
                _: &<Self::Reference as ReferenceStateMachine>::State,
            ) {
                // safety property for when we're using more than one CRDT here. Doing
                // this gives us a way to reason about which update happened first, as
                // well as letting us overcome clock drift.
                debug_assert!(
                    &state.clock >= state.document.minutes_per_ping.clock(),
                    "{} < {}",
                    state.clock,
                    state.document.minutes_per_ping.clock()
                );
                for (_, lww) in &state.document.tags {
                    debug_assert!(
                        &state.clock >= lww.clock(),
                        "{} < {}",
                        state.clock,
                        state.document.minutes_per_ping.clock()
                    );
                }
            }
        }

        prop_state_machine! {
            #[test]
            fn state_machine(sequential 1..20 => ReplicaStateMachine);
        }
    }
}
