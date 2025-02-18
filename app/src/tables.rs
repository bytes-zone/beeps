use anyhow::{Context, Error, Result};
use beeps_core::{document::Part, Hlc, Lww};
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name=crate::schema::minutes_per_pings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MinutesPerPing {
    pub minutes_per_ping: i32,
    pub timestamp: DateTime<Utc>,
    pub counter: i32,
    pub node: i32,
}

impl TryFrom<MinutesPerPing> for Part {
    type Error = Error;

    fn try_from(row: MinutesPerPing) -> Result<Part> {
        Ok(Self::MinutesPerPing(Lww::new(
            row.minutes_per_ping
                .try_into()
                .context("could not convert minutes_per_ping")?,
            Hlc::new_at(
                row.node.try_into().context("could not convert node")?,
                row.timestamp,
                row.counter
                    .try_into()
                    .context("could not convert counter")?,
            ),
        )))
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name=crate::schema::pings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Ping {
    pub id: i32,
    pub ping: DateTime<Utc>,
}

impl From<Ping> for Part {
    fn from(val: Ping) -> Self {
        Part::Ping(val.ping)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name=crate::schema::tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Tag {
    pub id: i32,
    pub ping: DateTime<Utc>,
    pub tag: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub counter: i32,
    pub node: i32,
}

impl TryFrom<Tag> for Part {
    type Error = Error;

    fn try_from(row: Tag) -> Result<Part> {
        Ok(Self::Tag((
            row.ping,
            Lww::new(
                row.tag,
                Hlc::new_at(
                    row.node.try_into().context("could not convert node")?,
                    row.timestamp,
                    row.counter
                        .try_into()
                        .context("could not convert counter")?,
                ),
            ),
        )))
    }
}
