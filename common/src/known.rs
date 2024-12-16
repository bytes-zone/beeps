use crate::merge::Merge;

/// A CRDT that allows many replicas to register an unknown value without
/// overwriting a known value. In practice, this behaves like a `Lww<Option<T>>`
/// but it can go from `None` to `Some` and never back.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum Known<T: Merge> {
    /// The value is unknown.
    #[cfg_attr(test, proptest(weight = 1))]
    Unknown,

    /// The value is known.
    #[cfg_attr(test, proptest(weight = 4))]
    Known(T),
}

impl<T: Merge> Merge for Known<T> {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (pick @ Self::Unknown, Self::Unknown) => pick,
            (Self::Unknown, pick @ Self::Known(_)) => pick,
            (pick @ Self::Known(_), Self::Unknown) => pick,
            (Self::Known(self_k), Self::Known(other_k)) => Self::Known(self_k.merge(other_k)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod merge {
        use super::*;
        use crate::lww::Lww;
        use proptest::proptest;

        proptest! {
            #[test]
            fn known_always_beats_unknown(other: Known<Lww<bool>>) {
                assert_eq!(Known::Unknown.merge(other.clone()), other)
            }

            #[test]
            fn known_values_merge(a: Lww<bool>, b: Lww<bool>) {
                assert_eq!(
                    Known::Known(a.clone()).merge(Known::Known(b.clone())),
                    Known::Known(a.merge(b))
                )
            }

            #[test]
            fn idempotent(a: Known<Lww<bool>>) {
                crate::merge::test_idempotent(a);
            }

            #[test]
            fn commutative(a: Known<Lww<bool>>, b: Known<Lww<bool>>) {
                crate::merge::test_commutative(a, b);
            }

            #[test]
            fn associative(a: Known<Lww<bool>>, b: Known<Lww<bool>>, c: Known<Lww<bool>>) {
                crate::merge::test_associative(a, b, c);
            }
        }
    }
}
