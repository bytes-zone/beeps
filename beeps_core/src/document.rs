use crate::gmap::GMap;
use crate::gset::GSet;
use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::merge::Merge;
use chrono::{DateTime, Utc};

/// The state that gets synced between replicas.
#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Document {
    /// The average number of minutes between each ping.
    pub minutes_per_ping: Lww<u16>,

    /// The pings that have been scheduled so far.
    #[cfg_attr(test, proptest(strategy = "pings()"))]
    pub pings: GSet<DateTime<Utc>>,

    /// The tag (if any) set for each ping.
    #[cfg_attr(test, proptest(strategy = "tags()"))]
    pub tags: GMap<DateTime<Utc>, Lww<Option<String>>>,
}

impl Document {
    /// Create a new, empty state. It has a default `minutes_per_ping`, but with
    /// a zero clock so that overwriting is always possible.
    pub fn new() -> Self {
        Self {
            minutes_per_ping: Lww::new(45, Hlc::zero()),
            pings: GSet::new(),
            tags: GMap::new(),
        }
    }

    /// Get the ping with the latest timestamp. Returns `None` if we have no
    /// pings.
    pub fn latest_ping(&self) -> Option<&DateTime<Utc>> {
        self.pings.iter().max()
    }

    /// Set the average number of minutes between pings going forward.
    pub fn set_minutes_per_ping(&mut self, new: u16, clock: Hlc) {
        self.minutes_per_ping.set(new, clock);
    }

    /// Add a ping, likely in coordination with a `Scheduler`.
    pub fn add_ping(&mut self, when: DateTime<Utc>) {
        self.pings.insert(when);
    }

    /// Tag an existing ping. Only allows you to tag pings that you know exist.
    /// If you need to get more pings, schedule them with `Scheduler` and
    /// `add_ping` first. `Replica` provides an easy way to coordinate this.
    pub fn tag_ping(&mut self, ping: DateTime<Utc>, tag: String, clock: Hlc) -> bool {
        if !self.pings.contains(&ping) {
            return false;
        }

        self.tags.upsert(ping, Lww::new(Some(tag), clock));
        true
    }

    /// Untag an existing ping. Like `tag_ping`, only allows you to untag pings
    /// that you know exist.
    pub fn untag_ping(&mut self, ping: DateTime<Utc>, clock: Hlc) -> bool {
        if !self.pings.contains(&ping) {
            return false;
        }

        self.tags.upsert(ping, Lww::new(None, clock));
        true
    }

    /// Get the tag (if any) for a given ping.
    pub fn get_tag(&self, ping: &DateTime<Utc>) -> Option<&String> {
        self.tags.get(ping).and_then(|l| l.value().as_ref())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Parts of the `State` that can be split and merged independently.
pub enum Part {
    /// A part to be applied to `minutes_per_ping`
    MinutesPerPing(Lww<u16>),

    /// A part to be applied to `pings`
    Ping(DateTime<Utc>),

    /// A part to be applied to `tags`
    Tag((DateTime<Utc>, Lww<Option<String>>)),
}

impl Merge for Document {
    type Part = Part;

    fn split(self) -> impl Iterator<Item = Self::Part> {
        self.minutes_per_ping
            .split()
            .map(Part::MinutesPerPing)
            .chain(self.pings.split().map(Part::Ping))
            .chain(self.tags.split().map(Part::Tag))
    }

    fn merge_part(&mut self, part: Part) {
        match part {
            Part::MinutesPerPing(part) => {
                self.minutes_per_ping.merge_part(part);
            }
            Part::Ping(part) => {
                self.pings.merge_part(part);
            }
            Part::Tag(part) => {
                self.tags.merge_part(part);
            }
        }
    }
}

#[cfg(test)]
proptest::prop_compose! {
    // TODO: we're going to all this hassle just to be able to use the timestamp
    // as a key. I'm not the happiest about that. Is there any way to make this
    // more succinct?
    fn pings()(items in proptest::collection::btree_set(crate::test::timestamp(), 1..5)) -> GSet<DateTime<Utc>> {
        GSet(items)
    }
}

#[cfg(test)]
proptest::prop_compose! {
    // Same here
    fn tags()(items in proptest::collection::hash_map(crate::test::timestamp(), proptest::prelude::any::<Lww<Option<String>>>(), 1..5)) -> GMap<DateTime<Utc>, Lww<Option<String>>> {
        GMap(items)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    mod merge {
        use super::*;

        proptest::proptest! {
            #[test]
            fn test_idempotent(a: Document) {
                crate::merge::test_idempotent(a);
            }

            #[test]
            fn test_commutative(a: Document, b: Document) {
                println!("{a:#?}");
                crate::merge::test_commutative(a, b);
            }

            #[test]
            fn test_associative(a: Document, b: Document, c: Document) {
                crate::merge::test_associative(a, b, c);
            }
        }
    }

    mod set_minutes_per_ping {
        use super::*;

        #[test]
        fn set() {
            let mut state = Document::new();

            state.set_minutes_per_ping(60, state.minutes_per_ping.clock().next());
            assert_eq!(*state.minutes_per_ping.value(), 60);
        }
    }

    mod add_ping {
        use super::*;

        #[test]
        fn add_ping() {
            let mut state = Document::new();

            let when = Utc::now();
            state.add_ping(when);
            assert!(state.pings.contains(&when));
        }
    }

    mod tag_ping {
        use super::*;

        #[test]
        fn when_ping_exists() {
            let mut state = Document::new();

            let when = Utc::now();
            state.add_ping(when);
            assert!(
                state.tag_ping(when, "test".to_string(), Hlc::zero()),
                "tagging did not succeed, but should have (ping existed)"
            );

            assert_eq!(state.get_tag(&when), Some(&"test".to_string()));
        }

        #[test]
        fn when_ping_does_not_exist() {
            let mut state = Document::new();

            assert!(
                !state.tag_ping(Utc::now(), "test".to_string(), Hlc::zero()),
                "tagging succeeded, but should not have (ping did not exist)"
            );
        }
    }

    mod state_machine {
        use super::*;
        use crate::NodeId;
        use proptest_state_machine::*;
        use std::collections::{HashMap, HashSet};

        #[derive(Debug, Clone)]
        enum Transition {
            SetMinutesPerPing(u16, Hlc),
            AddPing(chrono::DateTime<Utc>),
            TagPing(chrono::DateTime<Utc>, String, Hlc),
            UntagPing(chrono::DateTime<Utc>, Hlc),
        }

        #[derive(Debug, Clone)]
        struct RefState {
            node_id: NodeId,

            minutes_per_ping: u16,
            pings: HashSet<DateTime<Utc>>,
            tags: HashMap<DateTime<Utc>, String>,
        }

        impl ReferenceStateMachine for RefState {
            type State = RefState;

            type Transition = Transition;

            fn init_state() -> BoxedStrategy<Self::State> {
                any::<NodeId>()
                    .prop_map(|node_id| RefState {
                        node_id,

                        minutes_per_ping: 45,
                        pings: HashSet::new(),
                        tags: HashMap::new(),
                    })
                    .boxed()
            }

            fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
                let node_id = state.node_id;

                prop_oneof![
                    1 => (1..=4u16).prop_map(move |i| Transition::SetMinutesPerPing(i * 15, Hlc::new(node_id))),
                    10 => crate::test::timestamp_range(0..=2i64).prop_map(Transition::AddPing),
                    10 =>
                        (crate::test::timestamp_range(0..=2i64), "(a|b|c)")
                            .prop_map(move |(ts, tag)| Transition::TagPing(ts, tag, Hlc::new(node_id))),
                    5 =>
                        crate::test::timestamp_range(0..=2i64)
                            .prop_map(move |ts| Transition::UntagPing(ts, Hlc::new(node_id))),
                ]
                .boxed()
            }

            fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
                match transition {
                    Transition::SetMinutesPerPing(new, _) => {
                        state.minutes_per_ping = *new;
                    }
                    Transition::AddPing(when) => {
                        state.pings.insert(*when);
                    }
                    Transition::TagPing(when, tag, _) => {
                        state.tags.insert(*when, tag.clone());
                    }
                    Transition::UntagPing(when, _) => {
                        state.tags.remove(when);
                    }
                }

                state
            }
        }

        struct StateStateMachine {}

        impl StateMachineTest for StateStateMachine {
            type SystemUnderTest = Document;

            type Reference = RefState;

            fn init_test(
                _: &<Self::Reference as proptest_state_machine::ReferenceStateMachine>::State,
            ) -> Self::SystemUnderTest {
                Document::new()
            }

            fn apply(
                mut state: Self::SystemUnderTest,
                ref_state: &<Self::Reference as proptest_state_machine::ReferenceStateMachine>::State,
                transition: <Self::Reference as proptest_state_machine::ReferenceStateMachine>::Transition,
            ) -> Self::SystemUnderTest {
                match transition {
                    Transition::SetMinutesPerPing(new, clock) => {
                        state.set_minutes_per_ping(new, clock);

                        let actual = state.minutes_per_ping.value();
                        let reference = ref_state.minutes_per_ping;

                        assert_eq!(
                            actual, &reference,
                            "minutes_per_ping was not the same. Actual: `{actual}`, reference: `{reference}`"
                        );
                    }
                    Transition::AddPing(when) => {
                        state.add_ping(when);

                        let actual = state.pings.contains(&when);
                        let reference = ref_state.pings.contains(&when);

                        assert_eq!(actual, reference, "inconsistent ping {when}. Actual: `{actual}`, reference: `{reference}`");
                    }
                    Transition::TagPing(when, tag, clock) => {
                        if state.tag_ping(when, tag.clone(), clock) {
                            let actual = state.get_tag(&when);
                            let reference = ref_state.tags.get(&when);

                            assert_eq!(
                                actual,
                                reference,
                                "inconsistent tag for {when}. Actual: `{actual:?}`, reference: `{reference:?}`"
                            );
                        }
                    }
                    Transition::UntagPing(when, clock) => {
                        if state.untag_ping(when, clock) {
                            let actual = state.get_tag(&when);
                            let reference = ref_state.tags.get(&when);

                            assert_eq!(
                                actual,
                                reference,
                                "inconsistent tag for {when}. Actual: `{actual:?}`, reference: `{reference:?}`"
                            );
                        }
                    }
                }

                state
            }

            fn check_invariants(
                state: &Self::SystemUnderTest,
                _: &<Self::Reference as ReferenceStateMachine>::State,
            ) {
                // consistency property: if a ping is tagged, it must exist in the pings set as well
                for ping in state.tags.keys() {
                    assert!(
                        state.pings.contains(ping),
                        "tagged ping {ping} does not exist in pings set"
                    );
                }
            }
        }

        prop_state_machine! {
            #[test]
            fn state_machine(sequential 1..20 => StateStateMachine);
        }
    }
}
