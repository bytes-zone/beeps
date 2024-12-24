use clap::Parser;
use std::path::PathBuf;

/// A TUI for collecting and tagging pings
#[derive(Parser)]
#[clap(version)]
pub struct Config {
    /// Where should we store data?
    #[clap(long)]
    data_dir: Option<PathBuf>,
}

impl Config {
    /// Get either the configured or a default data directory. If no data
    /// directory can be found (e.g. because `$HOME` is unset) we will use the
    /// current directory.
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir
            .clone()
            .or_else(|| {
                directories::ProjectDirs::from("zone", "bytes", "beeps")
                    .map(|dirs| dirs.data_local_dir().to_owned())
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }
}
