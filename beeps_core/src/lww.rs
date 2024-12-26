use crate::hlc::Hlc;
use crate::merge::Merge;
use crate::split::Split;
use core::fmt::{self, Debug, Formatter};

/// A last-write-wins register. Values can be anything you like. We decide which
/// writes "win" when merging with a hybrid logical clock.
#[derive(PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Lww<T> {
    /// Any value we care to store.
    value: T,

    /// The clock used to figure out which write wins.
    clock: Hlc,
}

impl<T> Lww<T> {
    /// Create a new LWW register.
    pub fn new(value: T, clock: Hlc) -> Self {
        Self { value, clock }
    }

    /// Set the value of the register. If the clock is newer than the current
    /// clock, the write will be accepted.
    pub fn set(&mut self, value: T, clock: Hlc) {
        if clock > self.clock {
            self.value = value;
            self.clock = clock;
        }
    }

    /// Get the current value of the register.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get the current clock value guarding writes.
    pub fn clock(&self) -> &Hlc {
        &self.clock
    }
}

impl<T> Merge for Lww<T>
where
    T: Clone,
{
    fn merge(self, other: Self) -> Self {
        if other.clock > self.clock {
            other
        } else {
            self
        }
    }
}

impl<T> Split<Lww<T>> for Lww<T>
where
    T: Clone,
{
    type Part = Lww<T>;

    fn split(self) -> impl Iterator<Item = Self::Part> {
        std::iter::once(self)
    }

    fn merge_part(&mut self, part: Self::Part) {
        self.set(part.value, part.clock);
    }
}

impl<T: Debug> Debug for Lww<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Lww")
            .field("value", &self.value)
            .field("clock", &self.clock)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::proptest;

    #[test]
    fn overwrites_if_clock_is_newer() {
        let first_clock = Hlc::zero();

        let lww = Lww::new(1, first_clock).merge(Lww::new(2, first_clock.next()));

        assert_eq!(lww.value, 2);
    }

    #[test]
    fn rejects_if_clock_is_equal() {
        let first_clock = Hlc::zero();

        let lww = Lww::new(1, first_clock).merge(Lww::new(2, first_clock));

        assert_eq!(lww.value, 1);
    }

    #[test]
    fn rejects_if_clock_is_older() {
        let first_clock = Hlc::zero();

        let merged = Lww::new(1, first_clock.next()).merge(Lww::new(2, first_clock));

        assert_eq!(merged.value, 1);
    }

    proptest! {
        #[test]
        fn merge_commutative(a: Lww<bool>, b: Lww<bool>) {
            crate::merge::test_commutative(a, b);
        }

        #[test]
        fn merge_associative(a: Lww<bool>, b: Lww<bool>, c: Lww<bool>) {
            crate::merge::test_associative(a, b, c);
        }

        #[test]
        fn merge_idempotent(a: Lww<bool>) {
            crate::merge::test_idempotent(a);
        }
    }

    proptest! {
        #[test]
        fn merge_or_merge_parts(a: Lww<bool>, b: Lww<bool>) {
            crate::split::test_merge_or_merge_parts(a, b);
        }
    }
}
