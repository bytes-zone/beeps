use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand_pcg::Pcg32;

#[derive(Clone)]
pub struct Scheduler {
    average_pings_per_minute: f64,
    ping: DateTime<Utc>,
}

impl Scheduler {
    // only temporary in test-only
    #[cfg(test)]
    fn new(average_minutes_between_pings: u16, ping: DateTime<Utc>) -> Self {
        // We want to eventually find out how many minutes we should wait for the
        // next ping. To do that, we need to know the rate of pings per minute.
        let average_pings_per_minute = 1.0 / average_minutes_between_pings as f64;

        Self {
            average_pings_per_minute,
            ping,
        }
    }
}

impl Iterator for Scheduler {
    type Item = DateTime<Utc>;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn next(&mut self) -> Option<Self::Item> {
        // Next, we'll generate a random number based seeded with the time of the
        // last ping. We do this because it allows us to generate the same sequence,
        // no matter which node it comes from.
        let mut rng = Pcg32::new(
            // A Chrono timestamp is an i64. If that's a negative number (e.g.
            // before 1970) that will underflow to a very high u64 value. This seems
            // like it could cause a problem, but is actually fine—we're just using
            // this as a seed, so we can accept whatever behavior we like *as long
            // as it's consistent*.
            self.ping.timestamp() as u64,
            0xa02_bdbf_7bb3_c0a7, // Default stream
        );

        // We want an exponential distribution of values (many small values with a
        // few much longer ones.) To get there, we'll start with a uniform
        // distribution and use inverse transform sampling to transform it into what
        // we want.
        let uniform: f64 = rng.gen(); // 0.0f64..1.0f64
        let exponential = uniform.ln() / -self.average_pings_per_minute;

        // The exponential distribution above gives us fractional minutes. We'll
        // accept that fraction down to the second level.
        let adjustment_seconds = (exponential * 60.0).ceil() as i64;

        // and we're done! Our next value in the sequence is simply the last ping
        // plus the amount of seconds we just calculated.
        self.ping += Duration::seconds(adjustment_seconds);

        Some(self.ping)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    // The scheduler needs to be random, but consistent over time. We don't
    // really care about the values here, just that we have a heads-up if the
    // generation changes in some way.
    #[test]
    fn well_known_values() {
        let scheduler = Scheduler::new(45, Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());

        let dates = scheduler.take(5).collect::<Vec<_>>();
        let expected = vec![
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 17, 29).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 56, 45).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 2, 19, 23).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 3, 28, 26).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 4, 20, 39).unwrap(),
        ];

        assert_eq!(dates, expected);
    }

    proptest! {
        #[test]
        fn next_is_later_than_last_ping(
            minutes_per_ping in 1..=60u16,
            last_timestamp in 0i64..2_000_000_000_000i64,
        ) {
            let last_ping = Utc.timestamp_opt(last_timestamp, 0).unwrap();
            let mut scheduler = Scheduler::new(minutes_per_ping, last_ping);

            prop_assert!(scheduler.next().unwrap() > last_ping);
        }

        #[test]
        fn average_is_close_to_lambda(
            minutes_per_ping in 1..60u16,
            last_timestamp in 0i64..2_000_000_000_000i64,
        ) {
            let scheduler = Scheduler::new(minutes_per_ping, Utc.timestamp_opt(last_timestamp, 0).unwrap());
            let scheduler_offset = scheduler.clone();

            let sample_size = 2_000;

            let total_minutes: i64 = scheduler
                .zip(scheduler_offset.skip(1))
                .take(sample_size)
                .map(|(a, b)| b - a)
                .sum::<chrono::Duration>()
                .num_minutes();

            let average = total_minutes as f64 / sample_size as f64;

            let diff = (average / minutes_per_ping as f64).abs() - 1.0;

            assert!(
                diff < 0.1,
                "{:0.2} was not close to {:0.2} ({:0.4})",
                average,
                minutes_per_ping,
                diff,
            );
        }
    }
}
