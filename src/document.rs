use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{self, Context},
    Result,
};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pings: Vec<Ping>,
    lambda: f64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            pings: Vec::new(),
            lambda: 45.0 / 60.0,
        }
    }
}

impl Document {
    pub fn fill(&mut self) -> Result<()> {
        if self.pings.is_empty() {
            self.pings.push(Ping::default());
        }

        let now = Utc::now();
        let mut current = self
            .pings
            .last()
            .expect("there to be at least one ping after backfilling");

        while current.time <= now {
            let mut gen = Pcg32::new(current.time.timestamp().try_into()?, 0xa02bdbf7bb3c0a7);
            let adjustment = (gen.next_u32() as f64 / u32::MAX as f64).ln() / self.lambda * -1.0;
            let delta = chrono::Duration::minutes((adjustment * 60.0).floor() as i64);

            let next = Ping {
                time: current.time + delta,
                tag: None,
            };
            self.pings.push(next);
            current = self
                .pings
                .last()
                .expect("there to be a last ping after pushing");
        }

        Ok(())
    }

    pub fn current_mut(&mut self) -> Option<&mut Ping> {
        let now = Utc::now();

        self.pings.iter_mut().rev().find(|p| p.time <= now)
    }

    pub fn future(&self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings.iter().rev().find(|p| p.time > now)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub time: DateTime<Utc>,
    pub tag: Option<String>,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            tag: None,
        }
    }
}
