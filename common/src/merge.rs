pub trait Merge {
    #[must_use]
    fn merge(self, other: Self) -> Self;
}

#[cfg(test)]
pub fn test_idempotent<T>(orig: T)
where
    T: Merge + Clone + PartialEq + std::fmt::Debug,
{
    let a1 = orig.clone();
    let a2 = orig.clone();

    let merged = a1.merge(a2);

    assert_eq!(merged, orig, "idempotency failure")
}

#[cfg(test)]
pub fn test_commutative<T>(m1: T, m2: T)
where
    T: Merge + Clone + PartialEq + std::fmt::Debug,
{
    let a1 = m1.clone();
    let a2 = m2.clone();
    let merged1 = a1.merge(a2);

    let merged2 = m1.merge(m2);

    assert_eq!(merged1, merged2, "commutativity failure")
}

#[cfg(test)]
pub fn test_associative<T>(m1: T, m2: T, m3: T)
where
    T: Merge + Clone + PartialEq + std::fmt::Debug,
{
    let a1 = m1.clone();
    let a2 = m2.clone();
    let a3 = m3.clone();
    let merged1 = a1.merge(a2).merge(a3);

    let merged2 = m1.merge(m2.merge(m3));

    assert_eq!(merged1, merged2, "associativity failure")
}
