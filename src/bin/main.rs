use beeps::document::{Document, TimestampedOp};
use chrono::{Local, Utc};
use clap::Parser;
use color_eyre::{
    eyre::{self, Context},
    Result,
};
use directories::ProjectDirs;

/// Keep track of what you're doing throughout the day by being annoyed by a robot.
#[derive(Parser, Debug)]
struct Cli {}

impl Cli {
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

        if !path.exists() {
            return Ok(Document::default());
        }

        let data = std::fs::read_to_string(path).wrap_err("could not read data")?;
        let ops = serde_json::from_str(&data).wrap_err("could not deserialize data")?;

        Ok(Document::from_ops(ops))
    }

    fn save(&self, document: &Vec<TimestampedOp>) -> Result<()> {
        let dirs = self.dirs()?;
        let path = dirs.data_dir().join("data.json");
        let data = serde_json::to_string(document).wrap_err("could not serialize data")?;

        std::fs::create_dir_all(dirs.data_dir()).wrap_err("could not create directory")?;
        std::fs::write(path, data).wrap_err("could not write data")?;

        Ok(())
    }

    fn run(&self) -> Result<()> {
        let mut loaded = self.read().wrap_err("could not load document")?;

        loop {
            loaded.fill();

            if let Some(time) = loaded.current().filter(|p| p.tag.is_none()).map(|p| p.time) {
                println!(
                    "What were you doing at {}?",
                    time.with_timezone(&Local).format("%-I:%M %p")
                );
                let mut tag = String::new();
                std::io::stdin().read_line(&mut tag)?;

                let trimmed = tag.trim();
                if !trimmed.is_empty() {
                    loaded.set_tag(&time, trimmed.to_string())?;
                }
            }

            self.save(loaded.ops()).wrap_err("could not save")?;

            // fill again, just in case we waited forever to fill out the current ping
            loaded.fill();

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

    let cli = Cli::parse();
    if let Err(err) = cli.run() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}
