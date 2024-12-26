/// Split a data structure into parts (for storage and syncing) and
/// merge them back together later.
pub trait Split
where
    Self: Sized,
{
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
    /// 2. `empty.merge_parts(a.to_parts())` should result in a value equal to
    ///    the original `a`.
    /// 3. `a.merge_parts(b.split())` should be the same as `a.merge(b)`.
    fn split(self) -> impl Iterator<Item = Self::Part>;

    /// Build a data structure from the given parts. (For example, this is used
    /// when we load data from the database.)
    fn merge_part(&mut self, part: Self::Part);

    /// Merge two `Merge`s into one. This happens when we sync state between
    /// replicas. In order for CRDT semantics to hold, this operation must be
    /// commutative, associative, and idempotent. There are tests to help
    /// guarantee this below.
    fn merge(&mut self, other: Self) {
        for part in other.split() {
            self.merge_part(part);
        }
    }
}

/// Test that a Merge implementation is idempotent (in other words, merging
/// multiple times should not change the state.)
#[cfg(test)]
pub fn test_idempotent<T>(base: T, part: T::Part)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
    T::Part: Clone,
{
    let mut once = base.clone();
    once.merge_part(part.clone());

    let mut twice = base.clone();
    twice.merge_part(part.clone());
    twice.merge_part(part.clone());

    assert_eq!(once, twice);
}

/// Test that the implementation is commutative (in other words, the order of
/// merges should not effect the final result.)
#[cfg(test)]
pub fn test_commutative<T>(base: T, part_a: T::Part, part_b: T::Part)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
    T::Part: Clone,
{
    let mut ab = base.clone();
    ab.merge_part(part_a.clone());
    ab.merge_part(part_b.clone());

    let mut ba = base.clone();
    ba.merge_part(part_b.clone());
    ba.merge_part(part_a.clone());

    assert_eq!(ab, ba);
}

/// Test that a Merge implementation is associative (in other words, the order
/// in which replicas are merged should not effect the final result.)
#[cfg(test)]
pub fn test_associative<T>(a: T, b: T, c: T)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
{
    let mut abc = a.clone();
    abc.merge(b.clone());
    abc.merge(c.clone());

    let mut a_bc = a;
    let mut bc = b;
    bc.merge(c);
    a_bc.merge(bc);

    assert_eq!(abc, a_bc);
}

/// Test that `merge` and `merge_parts` hold the proper relationship. That is:
///
///     a.merge(b)
///
/// Should give the same result as:
///
///     for part in b.split() {
///         a.merge_part(part)
///     }
///
/// This is only useful if `merge` is implemented separately from `merge_part`,
/// as the default implementation does essentially the second code sample.
#[cfg(test)]
pub fn test_merge_or_merge_parts<T>(a: T, b: T)
where
    T: Split + Clone + PartialEq + std::fmt::Debug,
{
    let mut merged = a.clone();
    merged.merge(b.clone());

    let mut from_parts = a;
    for part in b.split() {
        from_parts.merge_part(part);
    }

    assert_eq!(from_parts, merged);
}
