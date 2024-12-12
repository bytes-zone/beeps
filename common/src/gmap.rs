use crate::merge::Merge;
use std::collections::{
    hash_map::{Drain, Entry, Iter},
    HashMap,
};
use std::hash::Hash;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct GMap<K: Eq + Hash, V: Merge>(pub(crate) HashMap<K, V>);

impl<K, V> GMap<K, V>
where
    K: Eq + Hash,
    V: Merge,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.0.entry(key) {
            Entry::Occupied(entry) => {
                let (key, current) = entry.remove_entry();
                let next = current.merge(value);
                self.0.insert(key, next);
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        };
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    /// Private because we can't remove properties from the map. It behaves like
    /// a G-Set. We will need it to merge, though!
    fn drain(&mut self) -> Drain<'_, K, V> {
        self.0.drain()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> Merge for GMap<K, V>
where
    K: Eq + Hash,
    V: Merge,
{
    fn merge(mut self, mut other: Self) -> Self {
        for (k, v) in other.drain() {
            self.insert(k, v)
        }

        self
    }
}

impl<K, V> Default for GMap<K, V>
where
    K: Eq + Hash,
    V: Merge,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::fmt::Debug for GMap<K, V>
where
    K: Eq + Hash + std::fmt::Debug,
    V: Merge + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LwwMap").field("0", &self.0).finish()
    }
}

impl<K, V> PartialEq for GMap<K, V>
where
    K: Eq + Hash,
    V: Merge + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hlc::Hlc;
    use crate::lww::Lww;

    mod get {
        use super::*;

        #[test]
        fn get_nothing() {
            let map = GMap::<&str, Lww<i32>>::new();
            assert_eq!(map.get(&"foo"), None);
        }

        // a "get_something" test would duplicate "insert::can_insert_from_nothing"
    }

    mod insert {
        use proptest::prelude::*;

        use super::*;

        #[test]
        fn can_insert_from_nothing() {
            let mut map = GMap::<&str, Lww<i32>>::new();
            map.insert("test", Lww::new(1, Hlc::zero()));

            assert_eq!(map.get(&"test").unwrap().value(), &1);
        }

        proptest! {
            #[test]
            fn insert_follows_lww_rules(
                lww1: Lww<u8>,
                lww2: Lww<u8>,
            ) {
                prop_assume!(lww1.value() != lww2.value());

                let mut map = GMap::<&str, Lww<u8>>::new();

                map.insert("test", lww1.clone());
                map.insert("test", lww2.clone());

                let result = map.get(&"test").unwrap();

                prop_assert_eq!(result, &lww1.merge(lww2));
            }
        }
    }

    mod merge {
        use proptest::prelude::*;

        use super::*;

        #[test]
        fn merge_nothing() {
            let map1 = GMap::<&str, Lww<i32>>::new();
            let map2 = GMap::<&str, Lww<i32>>::new();

            let merged = map1.merge(map2);

            assert_eq!(merged.len(), 0);
        }

        #[test]
        fn retains_all_keys() {
            let mut map1 = GMap::<&str, Lww<u8>>::new();
            map1.insert("foo", Lww::new(1, Hlc::zero()));

            let mut map2 = GMap::<&str, Lww<u8>>::new();
            map2.insert("bar", Lww::new(2, Hlc::zero()));

            let merged = map1.merge(map2);

            assert_eq!(merged.get(&"foo").unwrap().value(), &1);
            assert_eq!(merged.get(&"bar").unwrap().value(), &2);
        }

        proptest! {
            #[test]
            fn merges_according_to_merge_semantics_of_value(
                lww1: Lww<u8>,
                lww2: Lww<u8>,
            ) {
                let mut map1 = GMap::<&str, Lww<u8>>::new();
                map1.insert("test", lww1.clone());

                let mut map2 = GMap::<&str, Lww<u8>>::new();
                map2.insert("test", lww2.clone());

                let merged_lww = lww1.merge(lww2);
                let merged_map = map1.merge(map2);

                let result = merged_map.get(&"test").unwrap();

                prop_assert_eq!(result, &merged_lww);
            }

            #[test]
            fn merge_idempotent(a: GMap<u8, Lww<u8>>) {
                crate::merge::test_idempotent(a);
            }

            #[test]
            fn merge_commutative(a: GMap<u8, Lww<u8>>, b: GMap<u8, Lww<u8>>) {
                crate::merge::test_commutative(a, b);
            }

            #[test]
            fn merge_associative(
                a: GMap<u8, Lww<u8>>,
                b: GMap<u8, Lww<u8>>,
                c: GMap<u8, Lww<u8>>,
            ) {
                crate::merge::test_associative(a, b, c);
            }
        }
    }
}
