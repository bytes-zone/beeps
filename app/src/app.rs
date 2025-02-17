use beeps_core::{merge::Merge, Document, NodeId, Replica};
use color_eyre::eyre::Result;
use diesel::prelude::*;
use std::path::{Path, PathBuf};

use crate::tables::{MinutesPerPing, Ping, Tag};

pub struct App {
    replica: Replica,
    data_dir: PathBuf,
}

impl App {
    pub fn load() -> Result<Self> {
        let data_dir = data_dir();

        let replica = App::load_replica(get_conn(&data_dir)?)?;

        Ok(App { data_dir, replica })
    }

    fn load_replica(mut conn: SqliteConnection) -> Result<Replica> {
        let mut doc = Document::default();

        {
            use crate::schema::minutes_per_pings::dsl::*;

            for row in minutes_per_pings
                .select(MinutesPerPing::as_select())
                .load_iter(&mut conn)?
            {
                doc.merge_part(row?.try_into()?)
            }
        }

        {
            use crate::schema::pings::dsl::*;

            for row in pings.select(Ping::as_select()).load_iter(&mut conn)? {
                doc.merge_part(row?.try_into()?)
            }
        }

        {
            use crate::schema::tags::dsl::*;

            for row in tags.select(Tag::as_select()).load_iter(&mut conn)? {
                doc.merge_part(row?.try_into()?)
            }
        }

        // TODO: persist node id, possibly in sqlite?
        // TODO: combine initialization?
        let mut replica = Replica::new(NodeId::random());
        replica.replace_doc(doc);

        Ok(replica)
    }

    pub fn document(&self) -> &Document {
        self.replica.document()
    }

    pub fn get_conn(&self) -> Result<SqliteConnection> {
        get_conn(&self.data_dir)
    }
}

fn get_conn(data_dir: &Path) -> Result<SqliteConnection> {
    Ok(SqliteConnection::establish(&format!(
        "sqlite://{}",
        data_dir.join("beeps.sqlite3").to_string_lossy()
    ))?)
}

/// Get the data directory for the app.
fn data_dir() -> PathBuf {
    std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            directories::ProjectDirs::from("zone", "bytes", "beeps")
                .map(|d| d.data_local_dir().to_owned())
                .unwrap_or_else(|| PathBuf::from("."))
        })
}
