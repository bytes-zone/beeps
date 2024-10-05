use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::{eyre, Result};
use directories::ProjectDirs;
use rand_core::RngCore;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
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
    fn fill(&mut self) -> Result<()> {
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
}

#[derive(Debug, Serialize, Deserialize)]
struct Ping {
    time: DateTime<Utc>,
    tag: Option<String>,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            tag: None,
        }
    }
}

/// Keep track of what you're doing throughout the day by being annoyed by a robot.
#[derive(Parser, Debug)]
struct CLI {}

impl CLI {
    fn dirs(&self) -> Result<ProjectDirs> {
        match ProjectDirs::from("zone", "bytes", "beeps") {
            Some(l) => Ok(l),
            None => Err(eyre::eyre!(
                "Could not find a suitable location to store data"
            )),
        }
    }

    fn read(&self) -> Result<Document> {
        let dirs = self.dirs()?;
        let path = dirs.data_dir().join("data.json");
        let data = std::fs::read_to_string(path)?;
        let document = serde_json::from_str(&data)?;

        Ok(document)
    }

    fn run(&self) -> Result<()> {
        let mut loaded = self.read().unwrap_or_default();

        loaded.fill()?;

        println!("loaded: {loaded:#?}");

        Ok(())
    }
}

fn main() {
    color_eyre::install().unwrap();

    let cli = CLI::parse();
    if let Err(err) = cli.run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
