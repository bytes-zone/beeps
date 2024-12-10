use crate::hlc::Hlc;
use chrono::{TimeZone, Utc};
use proptest::prop_compose;
use uuid::Uuid;

prop_compose! {
    pub fn clock()
        (uuid: u128, timestamp in 0i64..2_000_000_000i64) -> Hlc {
        Hlc::new_at(
            Uuid::from_u128(uuid),
            Utc.timestamp_opt(timestamp, 0).unwrap(),
        )
    }
}
