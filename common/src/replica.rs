use crate::gmap::GMap;
use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::node_id::NodeId;
use crate::state::State;
use chrono::{DateTime, Utc};

pub struct Replica {
    // for bookkeeping
    clock: Hlc,
    document: State,
}

impl Replica {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            clock: Hlc::new(node_id),
            document: State::default(),
        }
    }

    fn next_clock(&mut self) -> Hlc {
        self.clock.increment();
        self.clock.clone()
    }

    pub fn minutes_per_ping(&self) -> &f64 {
        self.document.minutes_per_ping.value()
    }

    pub fn set_minutes_per_ping(&mut self, new: f64) {
        let clock = self.next_clock();
        self.document.minutes_per_ping.set(new, clock);
    }

    pub fn pings(&self) -> &GMap<DateTime<Utc>, Lww<Option<String>>> {
        &self.document.pings
    }

    pub fn add_ping(&mut self, when: DateTime<Utc>) {
        let clock = self.next_clock();
        self.document.pings.insert(when, Lww::new(None, clock));
    }

    pub fn tag_ping(&mut self, when: DateTime<Utc>, tag: String) {
        let clock = self.next_clock();
        self.document.pings.insert(when, Lww::new(Some(tag), clock));
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use proptest::prelude::*;
    use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};

    use super::*;

    #[test]
    fn minutes_per_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        doc.set_minutes_per_ping(60.0);
        assert_eq!(*doc.minutes_per_ping(), 60.0);
    }

    #[test]
    fn add_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        assert_eq!(doc.pings().get(&when).map(|lww| lww.value()), Some(&None));
    }

    #[test]
    fn set_ping() {
        let node_id = NodeId::random();
        let mut doc = Replica::new(node_id);

        let when = Utc::now();
        doc.add_ping(when);
        doc.tag_ping(when, "test".to_string());
        assert_eq!(
            doc.pings().get(&when).and_then(|lww| lww.value().clone()),
            Some("test".to_string())
        );
    }

    // Property Test
    #[derive(Debug, Clone)]
    enum Transition {
        SetMinutesPerPing(f64),
        AddPing(chrono::DateTime<Utc>),
        TagPing(chrono::DateTime<Utc>, String),
    }

    #[derive(Debug, Clone)]
    struct RefState {
        minutes_per_ping: f64,
        pings: HashMap<DateTime<Utc>, Option<String>>,
    }

    impl ReferenceStateMachine for RefState {
        type State = RefState;

        type Transition = Transition;

        fn init_state() -> BoxedStrategy<Self::State> {
            Just(RefState {
                minutes_per_ping: 45.0,
                pings: HashMap::new(),
            })
            .boxed()
        }

        fn transitions(_: &Self::State) -> BoxedStrategy<Self::Transition> {
            prop_oneof![
                1 => (1.0..60.0).prop_map(Transition::SetMinutesPerPing),
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

                    assert_eq!(state.minutes_per_ping(), &ref_state.minutes_per_ping);
                }
                Transition::AddPing(when) => {
                    state.add_ping(when);

                    assert_eq!(
                        state.pings().get(&when).map(|lww| lww.value()),
                        ref_state.pings.get(&when)
                    );
                }
                Transition::TagPing(when, tag) => {
                    state.tag_ping(when, tag.clone());

                    assert_eq!(
                        state.pings().get(&when).map(|lww| lww.value()),
                        Some(&Some(tag))
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
                &state.clock >= state.document.minutes_per_ping.clock(),
                "{} < {}",
                state.clock,
                state.document.minutes_per_ping.clock()
            );
            for (_, lww) in state.document.pings.iter() {
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
