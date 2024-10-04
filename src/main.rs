use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::{eyre, Result};
use directories::ProjectDirs;
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

#[derive(Debug, Serialize, Deserialize)]
struct Ping {
    time: DateTime<Utc>,
    tag: Option<String>,
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
        let loaded = self.read().unwrap_or_default();

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
