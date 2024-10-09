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

    pub fn value(&self) -> &T {
        &self.value
    }
}
