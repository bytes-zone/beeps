pub trait Merge {
    fn merge(&self, other: &Self) -> Self;
}

#[cfg(test)]
pub fn test_idempotent<T>(orig: T)
where
    T: Merge + PartialEq + std::fmt::Debug,
{
    let merged = orig.merge(&orig);

    assert_eq!(merged, orig, "idempotency failure")
}

#[cfg(test)]
pub fn test_commutative<T>(m1: T, m2: T)
where
    T: Merge + PartialEq + std::fmt::Debug,
{
    let merged1 = m1.merge(&m2);
    let merged2 = m2.merge(&m1);

    assert_eq!(merged1, merged2, "commutativity failure")
}

#[cfg(test)]
pub fn test_associative<T>(m1: T, m2: T, m3: T)
where
    T: Merge + PartialEq + std::fmt::Debug,
{
    let merged1 = m1.merge(&m2).merge(&m3);
    let merged2 = m1.merge(&m2.merge(&m3));

    assert_eq!(merged1, merged2, "associativity failure")
}
