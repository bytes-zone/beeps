use chrono::{DateTime, TimeZone, Utc};

proptest::prop_compose! {
    pub fn timestamp()(unix in 0..2_000_000_000_000i64) -> DateTime<Utc> {
        Utc.timestamp_opt(unix, 0).unwrap()
    }
}
