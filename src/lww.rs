use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::hlc::Hlc;

/// A last-write-wins register. The lifecycle goes like this:
///
/// 1. Create a new LWW register with a default value. Initially, the register
///    will not have a timestamp. (That means all values need a default! Use
///    `Option` if you like.)
/// 2. Update the register with a new value and a timestamp. If the timestamp
///    is "later" than the current timestamp (or the current timestamp is
///    blank) then the new value will be used.
#[derive(Debug, Serialize, Deserialize)]
pub struct Lww<T> {
    value: T,
    timestamp: Option<Hlc>,
}

impl<T> Lww<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: None,
        }
    }

    pub fn update(&mut self, ts: &Hlc, value: T) {
        if self.timestamp.is_none() || ts > self.timestamp.as_ref().unwrap() {
            self.value = value;
            self.timestamp = Some(ts.clone());
        }
    }

    pub fn timestamp(&self) -> Option<&Hlc> {
        self.timestamp.as_ref()
    }
}

impl<T> Deref for Lww<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};

    use super::*;

    #[test]
    fn lifecycle() {
        let mut lww = Lww::new(0);

        assert_eq!(*lww, 0);

        lww.update(&Hlc::new(0), 1);

        assert_eq!(*lww, 1)
    }

    #[test]
    fn rejects_older_timestamp() {
        let mut lww = Lww::new("nothing");

        let now = Utc::now();
        let then = now - Duration::seconds(1);

        lww.update(&Hlc::new_at(0, now), "newer");
        lww.update(&Hlc::new_at(0, then), "older");

        assert_eq!(*lww, "newer")
    }
}
