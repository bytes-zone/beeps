use crate::hlc::Hlc;
use core::fmt::{self, Debug, Formatter};

#[derive(PartialEq, Eq, Clone)]
pub struct Lww<T> {
    value: T,
    clock: Hlc,
}

impl<T> Lww<T> {
    pub fn new(value: T, clock: Hlc) -> Self {
        Self { value, clock }
    }

    pub fn set(&mut self, value: T, clock: Hlc) -> &Self {
        if clock > self.clock {
            self.value = value;
            self.clock = clock;
        }

        self
    }

    pub fn merge(mut self, other: Lww<T>) -> Self {
        self.set(other.value, other.clock);
        self
    }

    pub fn value(&self) -> &T {
        &self.value
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
    use proptest::{prop_assert_eq, proptest};

    use crate::test_utils::clock;

    use super::*;

    #[test]
    fn overwrites_if_clock_is_newer() {
        let node = uuid::Uuid::nil();
        let first_clock = Hlc::new(node);

        let mut lww = Lww::new(1, first_clock.clone());
        lww.set(2, first_clock.next());

        assert_eq!(lww.value, 2);
    }

    #[test]
    fn rejects_if_clock_is_equal() {
        let node = uuid::Uuid::nil();
        let first_clock = Hlc::new(node);

        let mut lww = Lww::new(1, first_clock.clone());
        lww.set(2, first_clock.clone());

        assert_eq!(lww.value, 1);
    }

    #[test]
    fn rejects_if_clock_is_older() {
        let node = uuid::Uuid::nil();
        let first_clock = Hlc::new(node);

        let mut lww = Lww::new(1, first_clock.next());
        lww.set(2, first_clock);

        assert_eq!(lww.value, 1);
    }

    proptest! {
        #[test]
        fn merge_commutative(a in clock(), b in clock()) {
            let lww_a = Lww::new(1, a);
            let lww_b = Lww::new(1, b);

            let merge_ab = lww_a.clone().merge(lww_b.clone());
            let merge_ba = lww_b.clone().merge(lww_a.clone());

            prop_assert_eq!(merge_ab, merge_ba);
        }

        #[test]
        fn merge_associative(a in clock(), b in clock(), c in clock()) {
            let lww_a = Lww::new(1, a);
            let lww_b = Lww::new(1, b);
            let lww_c = Lww::new(1, c);

            let merge_ab = lww_a.clone().merge(lww_b.clone()).merge(lww_c.clone());
            let merge_abc = lww_a.clone().merge(lww_b.clone().merge(lww_c.clone()));

            prop_assert_eq!(merge_ab, merge_abc);
        }

        #[test]
        fn merge_idempotent(a in clock()) {
            let lww_a = Lww::new(1, a);

            let merge_aa = lww_a.clone().merge(lww_a.clone());

            prop_assert_eq!(merge_aa, lww_a);
        }
    }
}
