use crate::lww::Lww;
use crate::merge::Merge;
use std::collections::{
    hash_map::{Drain, Entry, Iter},
    HashMap,
};
use std::hash::Hash;

pub struct LwwMap<K, V> {
    inner: HashMap<K, Lww<V>>,
}

impl<K, V> LwwMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&Lww<V>> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: K, value: Lww<V>) {
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

    pub fn iter(&self) -> Iter<'_, K, Lww<V>> {
        self.inner.iter()
    }

    /// Private because we can't remove properties from the map. It behaves like
    /// a G-Set. We will need it to merge, though!
    fn drain(&mut self) -> Drain<'_, K, Lww<V>> {
        self.inner.drain()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> Merge for LwwMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn merge(mut self, mut other: Self) -> Self {
        for (k, v) in other.drain() {
            self.insert(k, v)
        }

        self
    }
}

impl<K, V> Default for LwwMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hlc::Hlc;
    use uuid::Uuid;

    mod get {
        use super::*;

        #[test]
        fn get_nothing() {
            let map = LwwMap::<&str, i32>::new();
            assert_eq!(map.get(&"foo"), None);
        }

        // a "get_something" test would duplicate "insert::can_insert_from_nothing"
    }

    mod insert {
        use crate::test_utils::clock;
        use proptest::{prop_assert_eq, proptest};

        use super::*;

        #[test]
        fn can_insert_from_nothing() {
            let mut map = LwwMap::<&str, i32>::new();
            map.insert("test", Lww::new(1, Hlc::new(Uuid::nil())));

            assert_eq!(map.get(&"test").unwrap().value(), &1);
        }

        proptest! {
            #[test]
            fn insert_follows_lww_rules(
                c1 in clock(),
                c2 in clock(),
            ) {
                let mut map = LwwMap::<&str, &str>::new();
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
        use crate::test_utils::clock;
        use proptest::{prop_assert_eq, proptest};

        use super::*;

        #[test]
        fn merge_nothing() {
            let map1 = LwwMap::<&str, i32>::new();
            let map2 = LwwMap::<&str, i32>::new();

            let merged = map1.merge(map2);

            assert_eq!(merged.len(), 0);
        }

        #[test]
        fn retains_all_keys() {
            let mut map1 = LwwMap::<&str, i32>::new();
            map1.insert("foo", Lww::new(1, Hlc::new(Uuid::nil())));

            let mut map2 = LwwMap::<&str, i32>::new();
            map2.insert("bar", Lww::new(2, Hlc::new(Uuid::nil())));

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
                let mut map1 = LwwMap::<&str, &str>::new();
                let lww1 = Lww::new("c1", c1.clone());
                map1.insert("test", lww1.clone());

                let mut map2 = LwwMap::<&str, &str>::new();
                let lww2 = Lww::new("c2", c2.clone());
                map2.insert("test", lww2.clone());

                let merged_lww = lww1.merge(lww2);
                let merged_map = map1.merge(map2);

                let result = merged_map.get(&"test").unwrap();

                prop_assert_eq!(result, &merged_lww);
            }
        }
    }
}
