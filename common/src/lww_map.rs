use crate::lww::Lww;
use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;

pub struct LwwMap<K, V> {
    inner: HashMap<K, Lww<V>>,
}

impl<K, V> LwwMap<K, V>
where
    K: Eq + Hash,
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
            Entry::Occupied(mut entry) => {
                entry.get_mut().merge(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        };
    }
}

impl<K, V> Default for LwwMap<K, V>
where
    K: Eq + Hash,
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
        use chrono::{TimeZone, Utc};
        use proptest::{prop_assert_eq, prop_compose, proptest};

        use super::*;

        #[test]
        fn can_insert_from_nothing() {
            let mut map = LwwMap::<&str, i32>::new();
            map.insert("test", Lww::new(1, Hlc::new(Uuid::nil())));

            assert_eq!(map.get(&"test").unwrap().value(), &1);
        }

        prop_compose! {
            fn clock()
                (uuid: u128, timestamp in 0i64..2_000_000_000i64) -> Hlc {
                Hlc::new_at(
                    Uuid::from_u128(uuid),
                    Utc.timestamp_opt(timestamp, 0).unwrap(),
                )
            }
        }

        proptest! {
            #[test]
            fn insert_follows_merge_precedence_rules(
                c1 in clock(),
                c2 in clock(),
            ) {
                let mut map = LwwMap::<&str, &str>::new();
                map.insert("test", Lww::new("c1", c1.clone()));
                map.insert("test", Lww::new("c2", c2.clone()));

                let result = map.get(&"test").unwrap();

                match c1.cmp(&c2) {
                    std::cmp::Ordering::Less => {
                        prop_assert_eq!(result.value(), &"c2");
                    }
                    std::cmp::Ordering::Equal => {
                        prop_assert_eq!(result.value(), &"c1");
                    }
                    std::cmp::Ordering::Greater => {
                        prop_assert_eq!(result.value(), &"c1");
                    }
                }
            }
        }
    }
}
