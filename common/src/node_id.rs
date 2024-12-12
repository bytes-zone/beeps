use chrono::Utc;
use rand::Rng;
use rand_pcg::Pcg32;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct NodeId(u16);

impl NodeId {
    pub fn random() -> Self {
        Self(
            Pcg32::new(
                Utc::now().timestamp() as u64, // Seed (underflow OK if we're somehow pre-1970)
                0xa02bdbf7bb3c0a7,             // Stream (default)
            )
            .gen(),
        )
    }

    pub fn min() -> Self {
        Self(u16::MIN)
    }

    pub fn max() -> Self {
        Self(u16::MAX)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn min_should_be_less_than_max() {
        // Other tests depend on this functionality. Even if it's a simple test,
        // it matters!
        assert!(NodeId::min() < NodeId::max());
    }
}
