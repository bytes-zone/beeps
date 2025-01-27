use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::{document::Part, merge::Merge, sync::pull, Document, Hlc, Lww};
use sqlx::{query, FromRow, Row};
use tokio_stream::StreamExt;

#[tracing::instrument]
pub async fn handler(Conn(mut conn): Conn, claims: Claims) -> Result<Json<pull::Resp>, Error> {
    let mut doc = Document::default();

    let mut minutes_per_pings = query(
        "SELECT minutes_per_ping, timestamp, counter, node FROM minutes_per_pings WHERE document_id = $1",
    ).bind(claims.document_id)
    .fetch(&mut *conn);

    while let Some(row) = minutes_per_pings.try_next().await? {
        let minutes_per_ping: u16 = row.get::<i32, &str>("minutes_per_ping").try_into()?;

        let clock = Hlc::from_row(&row)?;

        doc.merge_part(Part::MinutesPerPing(Lww::new(minutes_per_ping, clock)));
    }

    Ok(Json(doc))
}

#[cfg(test)]
mod test {}
