use crate::merge::Merge;

pub trait Split: Merge
where
    Self: Sized,
{
    /// Split this CRDT into multiple parts. We use this for delta-state sync as
    /// well as getting minimal parts for storage in a database.
    ///
    /// Implementations of this method should follow two principles:
    ///
    /// 1. `split` should return the smallest possible parts.
    /// 2. `split` should be the inverse of `merge`. That is, repeatedly
    ///    `merge`ing all the parts returned from `split` should give us the
    ///    original CRDT.
    ///
    /// (Property test helpers for both of these are provided in the module.)
    fn split(self) -> Vec<Self>;
}

/// Test that `split` returns the smallest possible parts.
#[cfg(test)]
pub fn test_split_minimal<T>(orig: T)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
{
    for part in orig.split() {
        assert_eq!(part.clone().split(), vec![part]);
    }
}

/// Test that `split` returns parts that can later be `merge`d back together.
#[cfg(test)]
pub fn test_split_merge<T>(orig: T)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
{
    let mut parts = orig.clone().split().into_iter();
    let mut rebuilt = parts.next().expect("split to return at least one part");

    for next in parts {
        rebuilt = rebuilt.merge(next)
    }

    assert_eq!(rebuilt, orig);
}
