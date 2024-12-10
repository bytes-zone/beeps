use crate::hlc::Hlc;
use core::fmt::{self, Debug, Formatter};

#[derive(PartialEq, Eq)]
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
}
