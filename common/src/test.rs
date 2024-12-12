use chrono::{DateTime, TimeZone, Utc};

proptest::prop_compose! {
    pub fn timestamp()(unix in 1_700_000_000..1_800_000_000_000i64) -> DateTime<Utc> {
        Utc.timestamp_opt(unix, 0).unwrap()
    }
}
