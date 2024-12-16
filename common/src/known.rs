use crate::merge::Merge;

/// A CRDT that allows many replicas to register an unknown value without
/// overwriting a known value. In practice, this behaves like a `Lww<Option<T>>`
/// but it can only go from `None` to `Some` and never back.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Known<T: Merge> {
    pub option: Option<T>,
}

impl<T: Merge> Known<T> {
    pub fn unknown() -> Self {
        Self { option: None }
    }

    pub fn known(value: T) -> Self {
        Self {
            option: Some(value),
        }
    }
}

impl<T: Merge> Merge for Known<T> {
    fn merge(self, other: Self) -> Self {
        Self {
            option: match (self.option, other.option) {
                (pick @ None, None) => pick,
                (None, pick @ Some(_)) => pick,
                (pick @ Some(_), None) => pick,
                (Some(self_k), Some(other_k)) => Some(self_k.merge(other_k)),
            },
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
                assert_eq!(Known::unknown().merge(other.clone()), other)
            }

            #[test]
            fn known_values_merge(a: Lww<bool>, b: Lww<bool>) {
                assert_eq!(
                    Known::known(a.clone()).merge(Known::known(b.clone())),
                    Known::known(a.merge(b))
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
