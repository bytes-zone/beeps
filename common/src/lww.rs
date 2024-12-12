use crate::{hlc::Hlc, merge::Merge};
use core::fmt::{self, Debug, Formatter};

#[derive(PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
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
    use crate::{node_id::NodeId, test_utils::clock};
    use proptest::proptest;

    #[test]
    fn overwrites_if_clock_is_newer() {
        let node = NodeId::min();
        let first_clock = Hlc::new(node);

        let lww = Lww::new(1, first_clock.clone()).merge(Lww::new(2, first_clock.next()));

        assert_eq!(lww.value, 2);
    }

    #[test]
    fn rejects_if_clock_is_equal() {
        let node = NodeId::min();
        let first_clock = Hlc::new(node);

        let lww = Lww::new(1, first_clock.clone()).merge(Lww::new(2, first_clock));

        assert_eq!(lww.value, 1);
    }

    #[test]
    fn rejects_if_clock_is_older() {
        let node = NodeId::min();
        let first_clock = Hlc::new(node);

        let merged = Lww::new(1, first_clock.next()).merge(Lww::new(2, first_clock));

        assert_eq!(merged.value, 1);
    }

    proptest! {
        #[test]
        fn merge_commutative(a in clock(), b in clock()) {
            crate::merge::test_commutative(
                Lww::new(1, a),
                Lww::new(1, b),
            )
        }

        #[test]
        fn merge_associative(a in clock(), b in clock(), c in clock()) {
            crate::merge::test_associative(
                Lww::new(1, a),
                Lww::new(1, b),
                Lww::new(1, c),
            )
        }

        #[test]
        fn merge_idempotent(a in clock()) {
            crate::merge::test_idempotent(Lww::new(1, a))
        }
    }
}
