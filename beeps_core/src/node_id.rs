use chrono::Utc;
use rand::Rng;
use rand_pcg::Pcg32;
use std::fmt::{self, Display};
use std::ops::Deref;

/// A unique identifier for a node in the network.
#[derive(
    Debug,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    sqlx::Type,
)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct NodeId(#[cfg_attr(test, proptest(strategy = "0..=3u16"))] pub u16);

impl NodeId {
    /// Get a random node ID based on the current time. When assigning, you
    /// should check to make sure that there are no clocks in the current state
    /// that match this ID. (Ideally you'd also check that there are no other
    /// replicas who haven't written yet, but that's generally too much to ask.)
    #[expect(clippy::cast_sign_loss)]
    pub fn random() -> Self {
        Self(
            Pcg32::new(
                Utc::now().timestamp() as u64, // Seed (we're OK with underflow if timestamp is somehow pre-1970)
                0xa02_bdbf_7bb3_c0a7,          // Stream (default)
            )
            .random(),
        )
    }

    /// The least possible `NodeId`.
    pub fn min() -> Self {
        Self(u16::MIN)
    }

    /// The greatest possible `NodeId`.
    pub fn max() -> Self {
        Self(u16::MAX)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for NodeId {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<i32> for NodeId {
    type Error = std::num::TryFromIntError;

    fn try_from(id: i32) -> Result<NodeId, Self::Error> {
        id.try_into().map(NodeId)
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
