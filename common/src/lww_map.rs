use crate::lww::Lww;
use std::collections::HashMap;
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
        self.inner.insert(key, value);
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
        use super::*;

        #[test]
        fn can_insert_from_nothing() {
            let mut map = LwwMap::<&str, i32>::new();
            map.insert("foo", Lww::new(1, Hlc::new(Uuid::nil())));

            assert_eq!(map.get(&"foo").unwrap().value(), &1);
        }
    }
}
