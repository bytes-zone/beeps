use crate::hlc::Hlc;
use crate::known::Known;
use crate::lww::Lww;
use crate::node_id::NodeId;
use crate::scheduler::Scheduler;
use crate::state::State;
use chrono::{DateTime, Utc};

/// The local state of a replica ("who am I" and "what do I know"). Reading the
/// state should be fairly straightforward.
pub struct Replica {
    /// The clock used to write. Should always be higher than any clock in state.
    clock: Hlc,

    /// Data that this replica will write to and sync with peers.
    state: State,
}

impl Replica {
    /// Create a new replica with the given node ID.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            clock: Hlc::new(node_id),
            state: State::default(),
        }
    }

    /// Increment the clock and get the next value you should use. Always use
    /// this when writing to ensure that the replica-level clock is the highest
    /// in the state.
    #[must_use]
    fn next_clock(&mut self) -> Hlc {
        self.clock.increment();
        self.clock.clone()
    }

    /// Read the current state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Set the average number of minutes between pings.
    pub fn set_minutes_per_ping(&mut self, new: u16) {
        let clock = self.next_clock();
        self.state.minutes_per_ping.set(new, clock);
    }

    /// Add a ping, likely in coordination with a `Scheduler`.
    pub fn add_ping(&mut self, when: DateTime<Utc>) {
        let clock = self.next_clock();
        self.state
            .pings
            .upsert(when, Known::new(Lww::new(None, clock)));
    }

    /// Tag an existing ping (although there are no guards against tagging a
    /// ping that does not exist!)
    pub fn tag_ping(&mut self, when: DateTime<Utc>, tag: String) {
        let clock = self.next_clock();
        self.state
            .pings
            .upsert(when, Known::new(Lww::new(Some(tag), clock)));
    }

    /// Does the same as `schedule_ping` but allows you to specify the cutoff.
    fn schedule_pings_with_cutoff(&mut self, cutoff: DateTime<Utc>) {
        let latest_ping = if let Some(ping) = self.state.latest_ping().copied() {
            ping
        } else {
            let now = Utc::now();
            self.state.pings.upsert(now, Known::unknown());

            now
        };

        let scheduler = Scheduler::new(*self.state.minutes_per_ping.value(), latest_ping);

        for next in scheduler {
            self.state.pings.upsert(next, Known::unknown());

            // accepting one past the cutoff gets us into the future
            if next > cutoff {
                break;
            }
        }
    }

    /// Schedule pings into the future. We don't just schedule *up to* the given
    /// time, but go one past that. That means that if the given time is the
    /// current time, we end up with the time we should next notify at.
    pub fn schedule_pings(&mut self) {
        self.schedule_pings_with_cutoff(Utc::now());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};
    use std::collections::HashMap;

    #[test]
    fn minutes_per_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        doc.set_minutes_per_ping(60);
        assert_eq!(*doc.state().minutes_per_ping.value(), 60);
    }

    #[test]
    fn add_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        assert_eq!(doc.state().pings.get(&when), Some(&Known::unknown()));
    }

    #[test]
    fn set_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        doc.tag_ping(when, "test".to_string());
        assert_eq!(
            doc.state()
                .pings
                .get(&when)
                .and_then(|k| k.option.clone())
                .and_then(|lww| lww.value().clone()),
            Some("test".to_string())
        );
    }

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
                    .keys()
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
                assert!(doc.state().pings.contains_key(&date));
            }
        }
    }

    // Big ol' property test for system properties
    #[derive(Debug, Clone)]
    enum Transition {
        SetMinutesPerPing(u16),
        AddPing(chrono::DateTime<Utc>),
        TagPing(chrono::DateTime<Utc>, String),
    }

    #[derive(Debug, Clone)]
    struct RefState {
        minutes_per_ping: u16,
        pings: HashMap<DateTime<Utc>, Option<String>>,
    }

    impl ReferenceStateMachine for RefState {
        type State = RefState;

        type Transition = Transition;

        fn init_state() -> BoxedStrategy<Self::State> {
            Just(RefState {
                minutes_per_ping: 45,
                pings: HashMap::new(),
            })
            .boxed()
        }

        fn transitions(_: &Self::State) -> BoxedStrategy<Self::Transition> {
            prop_oneof![
                1 => (1..=4u16).prop_map(|i| Transition::SetMinutesPerPing(i * 15)),
                10 => crate::test::timestamp_range(0..=2i64).prop_map(Transition::AddPing),
                10 =>
                    (crate::test::timestamp_range(0..=2i64), "(a|b|c)")
                        .prop_map(|(ts, tag)| Transition::TagPing(ts, tag)),
            ]
            .boxed()
        }

        fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
            match transition {
                Transition::SetMinutesPerPing(new) => {
                    state.minutes_per_ping = *new;
                }
                Transition::AddPing(when) => {
                    state.pings.insert(*when, None);
                }
                Transition::TagPing(when, tag) => {
                    state.pings.insert(*when, Some(tag.clone()));
                }
            }

            state
        }

        fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
            match transition {
                Transition::SetMinutesPerPing(_) => true,
                Transition::AddPing(when) => !state.pings.contains_key(when),
                Transition::TagPing(when, _) => state.pings.contains_key(when),
            }
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
            ref_state: &<Self::Reference as proptest_state_machine::ReferenceStateMachine>::State,
            transition: <Self::Reference as proptest_state_machine::ReferenceStateMachine>::Transition,
        ) -> Self::SystemUnderTest {
            match transition {
                Transition::SetMinutesPerPing(new) => {
                    state.set_minutes_per_ping(new);

                    assert_eq!(
                        state.state().minutes_per_ping.value(),
                        &ref_state.minutes_per_ping
                    );
                }
                Transition::AddPing(when) => {
                    state.add_ping(when);

                    assert_eq!(
                        state
                            .state()
                            .pings
                            .get(&when)
                            .and_then(|k| k.option.clone())
                            .map(|l| l.value().clone()),
                        ref_state.pings.get(&when).cloned()
                    );
                }
                Transition::TagPing(when, tag) => {
                    state.tag_ping(when, tag.clone());

                    assert_eq!(
                        state
                            .state()
                            .pings
                            .get(&when)
                            .and_then(|k| k.option.clone())
                            .map(|lww| lww.value().clone()),
                        Some(Some(tag))
                    );
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
                &state.clock >= state.state.minutes_per_ping.clock(),
                "{} < {}",
                state.clock,
                state.state.minutes_per_ping.clock()
            );
            for (_, lww_opt) in &state.state.pings {
                if let Some(lww) = &lww_opt.option {
                    debug_assert!(
                        &state.clock >= lww.clock(),
                        "{} < {}",
                        state.clock,
                        state.state.minutes_per_ping.clock()
                    );
                }
            }
        }
    }

    prop_state_machine! {
        #[test]
        fn state_machine(sequential 1..20 => ReplicaStateMachine);
    }
}
