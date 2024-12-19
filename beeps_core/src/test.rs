use std::ops::RangeInclusive;

use chrono::{DateTime, TimeZone, Utc};

proptest::prop_compose! {
    pub fn timestamp()(unix in 1_700_000_000..1_800_000_000_000i64) -> DateTime<Utc> {
        Utc.timestamp_opt(unix, 0).unwrap()
    }
}

proptest::prop_compose! {
    pub fn timestamp_range(r: RangeInclusive<i64>)(unix in r) -> DateTime<Utc> {
        Utc.timestamp_opt(unix, 0).unwrap()
    }
}
