use crate::lww::Lww;
use crate::merge::Merge;
use std::collections::{
    hash_map::{Entry, Iter},
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

    pub fn get(&mut self, key: &K) -> Option<&Lww<V>> {
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
            let mut map = LwwMap::<&str, i32>::new();
            assert_eq!(map.get(&"foo"), None);
        }

        // a "get_something" test would duplicate "set::can_insert_from_nothing"
    }

    mod set {
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
}
