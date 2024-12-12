use crate::merge::Merge;
use std::collections::{
    hash_map::{Drain, Entry, Iter},
    HashMap,
};
use std::hash::Hash;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GrowOnlyMap<K: Eq + Hash, V: Merge> {
    inner: HashMap<K, V>,
}

impl<K, V> GrowOnlyMap<K, V>
where
    K: Eq + Hash,
    V: Merge,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.inner.entry(key) {
            Entry::Occupied(entry) => {
                let (key, current) = entry.remove_entry();
                let next = current.merge(value);
                self.inner.insert(key, next);
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        };
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.inner.iter()
    }

    /// Private because we can't remove properties from the map. It behaves like
    /// a G-Set. We will need it to merge, though!
    fn drain(&mut self) -> Drain<'_, K, V> {
        self.inner.drain()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> Merge for GrowOnlyMap<K, V>
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

impl<K, V> Default for GrowOnlyMap<K, V>
where
    K: Eq + Hash,
    V: Merge,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::fmt::Debug for GrowOnlyMap<K, V>
where
    K: Eq + Hash + std::fmt::Debug,
    V: Merge + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LwwMap")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K, V> PartialEq for GrowOnlyMap<K, V>
where
    K: Eq + Hash,
    V: Merge + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
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
            let map = GrowOnlyMap::<&str, Lww<i32>>::new();
            assert_eq!(map.get(&"foo"), None);
        }

        // a "get_something" test would duplicate "insert::can_insert_from_nothing"
    }

    mod insert {
        use crate::{node_id::NodeId, test_utils::clock};
        use proptest::{prop_assert_eq, proptest};

        use super::*;

        #[test]
        fn can_insert_from_nothing() {
            let mut map = GrowOnlyMap::<&str, Lww<i32>>::new();
            map.insert("test", Lww::new(1, Hlc::new(NodeId::min())));

            assert_eq!(map.get(&"test").unwrap().value(), &1);
        }

        proptest! {
            #[test]
            fn insert_follows_lww_rules(
                c1 in clock(),
                c2 in clock(),
            ) {
                let mut map = GrowOnlyMap::<&str, Lww<&str>>::new();
                let lww1 = Lww::new("c1", c1.clone());
                let lww2 = Lww::new("c2", c2.clone());

                map.insert("test", lww1.clone());
                map.insert("test", lww2.clone());

                let result = map.get(&"test").unwrap();

                prop_assert_eq!(result, &lww1.merge(lww2));
            }
        }
    }

    mod merge {
        use crate::{node_id::NodeId, test_utils::clock};
        use proptest::{prop_assert_eq, proptest};

        use super::*;

        #[test]
        fn merge_nothing() {
            let map1 = GrowOnlyMap::<&str, Lww<i32>>::new();
            let map2 = GrowOnlyMap::<&str, Lww<i32>>::new();

            let merged = map1.merge(map2);

            assert_eq!(merged.len(), 0);
        }

        #[test]
        fn retains_all_keys() {
            let mut map1 = GrowOnlyMap::<&str, Lww<i32>>::new();
            map1.insert("foo", Lww::new(1, Hlc::new(NodeId::min())));

            let mut map2 = GrowOnlyMap::<&str, Lww<i32>>::new();
            map2.insert("bar", Lww::new(2, Hlc::new(NodeId::min())));

            let merged = map1.merge(map2);

            assert_eq!(merged.get(&"foo").unwrap().value(), &1);
            assert_eq!(merged.get(&"bar").unwrap().value(), &2);
        }

        proptest! {
            #[test]
            fn merges_according_to_merge_semantics_of_value(
                c1 in clock(),
                c2 in clock(),
            ) {
                let mut map1 = GrowOnlyMap::<&str, Lww<&str>>::new();
                let lww1 = Lww::new("c1", c1.clone());
                map1.insert("test", lww1.clone());

                let mut map2 = GrowOnlyMap::<&str, Lww<&str>>::new();
                let lww2 = Lww::new("c2", c2.clone());
                map2.insert("test", lww2.clone());

                let merged_lww = lww1.merge(lww2);
                let merged_map = map1.merge(map2);

                let result = merged_map.get(&"test").unwrap();

                prop_assert_eq!(result, &merged_lww);
            }

            #[test]
            fn merge_idempotent(
                c1 in clock(),
            ) {
                let mut map = GrowOnlyMap::<&str, Lww<&str>>::new();
                map.insert("test", Lww::new("c1", c1));

                crate::merge::test_idempotent(map);
            }

            #[test]
            fn merge_commutative(
                c1 in clock(),
                c2 in clock(),
            ) {
                let mut map1 = GrowOnlyMap::<&str, Lww<&str>>::new();
                map1.insert("test", Lww::new("c1", c1));

                let mut map2 = GrowOnlyMap::<&str, Lww<&str>>::new();
                map2.insert("test", Lww::new("c2", c2));

                crate::merge::test_commutative(map1, map2);
            }

            #[test]
            fn merge_associative(
                c1 in clock(),
                c2 in clock(),
                c3 in clock(),
            ) {
                let mut map1 = GrowOnlyMap::<&str, Lww<&str>>::new();
                map1.insert("test", Lww::new("c1", c1));

                let mut map2 = GrowOnlyMap::<&str, Lww<&str>>::new();
                map2.insert("test", Lww::new("c2", c2));

                let mut map3 = GrowOnlyMap::<&str, Lww<&str>>::new();
                map3.insert("test", Lww::new("c3", c3));

                crate::merge::test_associative(map1, map2, map3);
            }
        }
    }
}
