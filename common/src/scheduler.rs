use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand_pcg::Pcg32;

pub fn next(average_minutes_between_pings: f64, last_ping: DateTime<Utc>) -> DateTime<Utc> {
    // We want to eventually find out how many minutes we should wait for the
    // next ping, so the first thing we need to do is get rate of the number of
    // pings per minute.
    let average_pings_per_minute = 1.0 / average_minutes_between_pings;

    // Next, we'll generate a random number based seeded with the time of the
    // last ping. We do this because it allows us to generate the same sequence,
    // no matter which node it comes from.
    let mut rng = Pcg32::new(
        // A Chrono timestamp is an i64. If that's a negative number (e.g.
        // before 1970) that will underflow to a very high u64 value. This seems
        // like it could cause a problem, but is actually fine—we're just using
        // this as a seed, so we can accept whatever behavior we like *as long
        // as it's consistent*.
        last_ping.timestamp() as u64,
        0xa02bdbf7bb3c0a7, // Default stream
    );

    // We want an exponential distribution of values (many small values with a
    // few much longer ones.) To get there, we'll start with a uniform
    // distribution and use inverse transform sampling to transform it into what
    // we want.
    let uniform: f64 = rng.gen(); // 0.0f64..1.0f64
    let exponential = uniform.ln() / -average_pings_per_minute;

    // The exponential distribution above gives us fractional minutes. We'll
    // accept that fraction down to the second level.
    let adjustment_seconds = (exponential * 60.0).ceil() as i64;

    // and we're done! Our next value in the sequence is simply the last ping
    // plus the amount of seconds we just calculated.
    last_ping + Duration::seconds(adjustment_seconds)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn next_is_later_than_last_ping(
            lambda in 0.0..1.0,
            last_timestamp in 0i64..2_000_000_000_000i64,
        ) {
            let last_ping = Utc.timestamp_opt(last_timestamp, 0).unwrap();
            let next = super::next(lambda, last_ping);
            prop_assert!(next > last_ping);
        }
    }
}
