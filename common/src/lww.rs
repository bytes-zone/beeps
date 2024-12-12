use crate::{hlc::Hlc, merge::Merge};
use core::fmt::{self, Debug, Formatter};

#[derive(PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Lww<T> {
    value: T,
    clock: Hlc,
}

impl<T> Lww<T> {
    pub fn new(value: T, clock: Hlc) -> Self {
        Self { value, clock }
    }

    pub fn set(&mut self, value: T, clock: Hlc) {
        if clock > self.clock {
            self.value = value;
            self.clock = clock;
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

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

        let lww = Lww::new(1, first_clock.clone()).merge(Lww::new(2, first_clock.next()));

        assert_eq!(lww.value, 2);
    }

    #[test]
    fn rejects_if_clock_is_equal() {
        let first_clock = Hlc::zero();

        let lww = Lww::new(1, first_clock.clone()).merge(Lww::new(2, first_clock));

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
            crate::merge::test_commutative(a, b)
        }

        #[test]
        fn merge_associative(a: Lww<bool>, b: Lww<bool>, c: Lww<bool>) {
            crate::merge::test_associative(a, b, c)
        }

        #[test]
        fn merge_idempotent(a: Lww<bool>) {
            crate::merge::test_idempotent(a)
        }
    }
}
