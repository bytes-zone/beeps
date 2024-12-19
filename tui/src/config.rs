use clap::Parser;
use std::path::PathBuf;

/// A TUI for collecting and tagging pings
#[derive(Parser)]
pub struct Config {
    /// Where should we store data?
    #[clap(long)]
    data_dir: Option<PathBuf>,
}

impl Config {
    fn data_dir(&self) -> PathBuf {
        self.data_dir
            .clone()
            .or_else(|| {
                directories::ProjectDirs::from("zone", "bytes", "beeps")
                    .map(|dirs| dirs.data_local_dir().to_owned())
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }
}
