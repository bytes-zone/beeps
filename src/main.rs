use chrono::{DateTime, Local, Utc};
use clap::Parser;
use color_eyre::{
    eyre::{self, Context},
    Result,
};
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

    fn current_mut(&mut self) -> Option<&mut Ping> {
        let now = Utc::now();

        self.pings.iter_mut().rev().filter(|p| p.time <= now).next()
    }

    fn future(&self) -> Option<&Ping> {
        let now = Utc::now();

        self.pings.iter().rev().filter(|p| p.time > now).next()
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
        let data = std::fs::read_to_string(path).wrap_err("could not read data")?;
        let document = serde_json::from_str(&data).wrap_err("could not deserialize data")?;

        Ok(document)
    }

    fn save(&self, document: &Document) -> Result<()> {
        let dirs = self.dirs()?;
        let path = dirs.data_dir().join("data.json");
        let data = serde_json::to_string(document).wrap_err("could not serialize data")?;

        std::fs::create_dir_all(dirs.data_dir()).wrap_err("could not create directory")?;
        std::fs::write(path, data).wrap_err("could not write data")?;

        Ok(())
    }

    fn run(&self) -> Result<()> {
        let mut loaded = self.read().unwrap_or_default();

        loop {
            loaded.fill().wrap_err("could not fill")?;

            if let Some(ping) = loaded.current_mut().filter(|p| p.tag.is_none()) {
                println!(
                    "What were you doing at {}?",
                    ping.time.with_timezone(&Local).format("%-I:%M %p")
                );
                let mut tag = String::new();
                std::io::stdin().read_line(&mut tag)?;
                ping.tag = Some(tag.trim().to_string());
            }

            self.save(&loaded).wrap_err("could not save")?;

            // fill again, just in case we waited forever to fill out the current ping
            loaded.fill().wrap_err("could not fill")?;

            if let Some(ping) = loaded.future() {
                let now = Utc::now();
                let duration = ping.time - now;
                let duration = duration
                    .to_std()
                    .wrap_err("could not convert duration to std")?;
                println!("waiting for next ping");
                std::thread::sleep(duration);

                std::process::Command::new("say")
                    .arg("you have a new ping")
                    .spawn()
                    .wrap_err("could not invoke say")?;
            } else {
                break;
            }
        }

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
