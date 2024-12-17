use crate::merge::Merge;
use core::fmt;
use std::collections::{hash_set, HashSet};
use std::hash::Hash;
use std::iter::Extend;

/// A Grow-Only Set (G-Set) CRDT.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct GSet<T: Eq + Hash> {
    /// The items in the set. Should only be added to while in use.
    pub(crate) items: HashSet<T>,
}

impl<T: Eq + Hash> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> GSet<T> {
    /// Creates an empty `GSet`
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }

    /// An iterator visiting all elements in arbitrary order.
    pub fn iter(&self) -> hash_set::Iter<'_, T> {
        self.items.iter()
    }

    /// Adds a value to the set. Returns whether or not the value was newly
    /// inserted.
    pub fn insert(&mut self, value: T) -> bool {
        self.items.insert(value)
    }

    /// Returns true if the set contains the value.
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        self.items.contains(value)
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: Eq + Hash> Merge for GSet<T> {
    fn merge(mut self, mut other: Self) -> Self {
        self.items.extend(other.items.drain());

        self
    }
}

impl<T> fmt::Debug for GSet<T>
where
    T: Eq + Hash + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GSet").field("items", &self.items).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_idempotent(a: GSet<u8>) {
                crate::merge::test_idempotent(a);
            }

            #[test]
            fn test_commutative(a: GSet<u8>, b: GSet<u8>) {
                crate::merge::test_commutative(a, b);
            }

            #[test]
            fn test_associative(a: GSet<u8>, b: GSet<u8>, c: GSet<u8>) {
                crate::merge::test_associative(a, b, c);
            }
        }
    }
}
