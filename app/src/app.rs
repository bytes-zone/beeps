use beeps_core::{Document, NodeId, Replica};
use color_eyre::eyre::{Context, Result};
use std::path::{Path, PathBuf};

pub struct App {
    replica: Replica,
}

impl App {
    pub fn load() -> Result<Self> {
        let data_dir = data_dir();
        let replica = App::load_replica(&data_dir)?;

        Ok(App { replica })
    }

    fn load_replica(base: &Path) -> Result<Replica> {
        let store_path = base.join("store.json");

        if store_path.exists() {
            let file = std::fs::File::open(&store_path)?;
            serde_json::from_reader(file).wrap_err("could not read saved store")
        } else {
            Ok(Replica::new(NodeId::random()))
        }
    }

    pub fn document(&self) -> &Document {
        self.replica.document()
    }
}

/// Get the data directory for the app.
fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("zone", "bytes", "beeps")
        .map(|d| d.data_local_dir().to_owned())
        .unwrap_or_else(|| PathBuf::from("."))
}
