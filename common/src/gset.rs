use crate::merge::Merge;
use core::fmt;
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::Extend;

/// A Grow-Only Set (G-Set) CRDT.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct GSet<T: Eq + Hash> {
    /// The items in the set. Should only be added to while in use.
    items: HashSet<T>,
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
