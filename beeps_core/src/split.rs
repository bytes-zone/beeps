use crate::merge::Merge;

/// Split a data structure into parts (for storage and syncing) and
/// merge them back together later.
pub trait Split<Part>: Merge {
    /// The "parts" that we split this data structure into. These can be
    /// whatever you like, but should generally be the smallest parts possible.
    type Part;

    /// Split this data structure into multiple parts. We use this for
    /// delta-state sync as well as getting minimal parts for storage in a
    /// database.
    ///
    /// Implementations of this method should follow two principles:
    ///
    /// 1. `to_parts` should return the smallest possible parts.
    /// 2. `to_parts` should be the inverse of `merge_parts` (given an empty
    ///    data structure to start.)
    /// 3. For implementors that also implement `Merge`, `merge_parts` should
    ///    give the equivalent of `merge`.
    ///
    /// (Property test helpers for both of these are provided in the module.)
    fn split(self) -> Vec<Part>;

    /// Build a data structure from the given parts. (For example, this is used
    /// when load data from the database.)
    fn merge_parts(&mut self, parts: Vec<Part>);
}

/// Test that `split` returns the smallest possible parts.
// #[cfg(test)]
// pub fn test_split_minimal<T, Part>(orig: T)
// where
//     T: Parts<Part> + Clone + PartialEq + std::fmt::Debug,
// {
//     for part in orig.to_parts() {
//         assert_eq!(part.clone().split(), vec![part]);
//     }
// }

/// Test the relationship between `merge_parts` and `merge`. That is, for any
/// data structure that implements `Merge`, merging the parts of one data
/// structure into the other should give the same results as merging the two
/// data structures directly.
#[cfg(test)]
pub fn test_merge_or_merge_parts<T, Part>(a: T, b: T)
where
    T: Split<Part> + Clone + PartialEq + std::fmt::Debug,
{
    let merged = a.clone().merge(b.clone());

    let mut from_parts = a;
    from_parts.merge_parts(b.split());

    assert_eq!(from_parts, merged);
}
