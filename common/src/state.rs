use crate::hlc::Hlc;
use crate::lww::Lww;
use crate::merge::Merge;
use crate::{gmap::GMap, gset::GSet};
use chrono::{DateTime, Utc};

/// The state that gets synced between replicas.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct State {
    /// The average number of minutes between each ping.
    pub minutes_per_ping: Lww<u16>,

    /// The pings that have been filled into this struct.
    #[cfg_attr(test, proptest(strategy = "pings()"))]
    pub pings: GSet<DateTime<Utc>>,

    /// The tag (if any) set for each ping.
    #[cfg_attr(test, proptest(strategy = "tags()"))]
    pub tags: GMap<DateTime<Utc>, Lww<String>>,
}

impl State {
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
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl Merge for State {
    fn merge(mut self, other: Self) -> Self {
        self.minutes_per_ping = self.minutes_per_ping.merge(other.minutes_per_ping);
        self.pings = self.pings.merge(other.pings);

        self
    }
}

#[cfg(test)]
proptest::prop_compose! {
    // TODO: we're going to all this hassle just to be able to use the timestamp
    // as a key. I'm not the happiest about that. Is there any way to make this
    // more succinct?
    fn pings()(items in proptest::collection::btree_set(crate::test::timestamp(), 1..5)) -> GSet<DateTime<Utc>> {
        GSet { items }
    }
}

#[cfg(test)]
proptest::prop_compose! {
    // Same here
    fn tags()(items in proptest::collection::hash_map(crate::test::timestamp(), proptest::prelude::any::<Lww<String>>(), 1..5)) -> GMap<DateTime<Utc>, Lww<String>> {
        GMap(items)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_merge_idempotent(a: State) {
            crate::merge::test_idempotent(a);
        }

        #[test]
        fn test_merge_commutative(a: State, b: State) {
            println!("{a:#?}");
            crate::merge::test_commutative(a, b);
        }

        #[test]
        fn test_merge_associative(a: State, b: State, c: State) {
            crate::merge::test_associative(a, b, c);
        }
    }
}
