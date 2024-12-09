use crate::hlc::Hlc;

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
