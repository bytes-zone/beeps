use crate::merge::Merge;
use crate::split::Split;
use core::fmt;
use std::collections::{btree_set, BTreeSet};

/// A Grow-Only Set (G-Set) CRDT.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct GSet<T: Ord>(pub(crate) BTreeSet<T>);

impl<T: Ord> GSet<T> {
    /// Creates an empty `GSet`
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// An iterator visiting all elements in arbitrary order.
    pub fn iter(&self) -> btree_set::Iter<'_, T> {
        self.0.iter()
    }

    /// Adds a value to the set. Returns whether or not the value was newly
    /// inserted.
    pub fn insert(&mut self, value: T) -> bool {
        self.0.insert(value)
    }

    /// Returns true if the set contains the value.
    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Ord> Merge for GSet<T> {
    fn merge(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);

        self
    }
}

impl<T> fmt::Debug for GSet<T>
where
    T: Ord + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GSet").field("items", &self.0).finish()
    }
}

impl<T: Ord> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: Ord> IntoIterator for &'a GSet<T> {
    type IntoIter = btree_set::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Split<T> for GSet<T>
where
    T: Ord,
{
    type Part = T;

    fn split(self) -> impl Iterator<Item = Self::Part> {
        self.0.into_iter()
    }

    fn merge_part(&mut self, part: Self::Part) {
        self.insert(part);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod merge {
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

    mod split {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn merge_or_merge_parts(a: GSet<u8>, b: GSet<u8>) {
                crate::split::test_merge_or_merge_parts(a, b);
            }
        }
    }
}
