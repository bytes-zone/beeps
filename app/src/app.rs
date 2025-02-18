use crate::tables::{MinutesPerPing, Ping, Tag};
use anyhow::{Context, Error, Result};
use beeps_core::{merge::Merge, Document, NodeId, Replica};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::path::{Path, PathBuf};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct App {
    replica: Replica,
    database_url: String,
}

impl App {
    pub fn load(database_url: &str) -> Result<Self> {
        let mut conn = get_conn(&database_url).context("could not get connection")?;

        conn.run_pending_migrations(MIGRATIONS)
            .map_err(Error::from_boxed)
            .context("could not run migrations")?;

        let replica =
            App::load_replica(&mut conn).context("could not load replica from database")?;

        Ok(App {
            database_url: database_url.to_owned(),
            replica,
        })
    }

    fn load_replica(conn: &mut SqliteConnection) -> Result<Replica> {
        let mut doc = Document::default();

        {
            use crate::schema::minutes_per_pings::dsl::*;

            for row in minutes_per_pings
                .select(MinutesPerPing::as_select())
                .load_iter(conn)?
            {
                doc.merge_part(row?.try_into()?)
            }
        }

        {
            use crate::schema::pings::dsl::*;

            for row in pings.select(Ping::as_select()).load_iter(conn)? {
                doc.merge_part(row?.into())
            }
        }

        {
            use crate::schema::tags::dsl::*;

            for row in tags.select(Tag::as_select()).load_iter(conn)? {
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
        get_conn(&self.database_url)
    }
}

fn get_conn(database_url: &str) -> Result<SqliteConnection> {
    SqliteConnection::establish(database_url).context("could not establish connection")
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

/// Get the data directory for the app.
pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "sqlite://{}",
            data_dir().join("beeps.sqlite3").to_string_lossy()
        )
    })
}
