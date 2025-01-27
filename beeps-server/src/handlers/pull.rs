use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::{document::Part, merge::Merge, sync::pull, Document, Hlc, Lww};
use chrono::{DateTime, Utc};
use sqlx::{query_as, FromRow};
use tokio_stream::StreamExt;

#[tracing::instrument]
pub async fn handler(Conn(mut conn): Conn, claims: Claims) -> Result<Json<pull::Resp>, Error> {
    let mut doc = Document::default();

    // minutes per ping
    {
        let mut minutes_per_pings = query_as!(
            MinutesPerPingRow,
            "SELECT minutes_per_ping, timestamp, counter, node FROM minutes_per_pings WHERE document_id = $1",
            claims.document_id,
        )
        .fetch(&mut *conn);

        while let Some(row) = minutes_per_pings.try_next().await? {
            doc.merge_part(row.try_into()?);
        }
    }

    // pings
    {
        let mut pings = query_as!(
            PingRow,
            "SELECT ping FROM pings WHERE document_id = $1",
            claims.document_id
        )
        .fetch(&mut *conn);

        while let Some(row) = pings.try_next().await? {
            doc.merge_part(row.into());
        }
    }

    // tags
    {
        let mut tags = query_as!(
            TagRow,
            "SELECT ping, tag, timestamp, counter, node FROM tags WHERE document_id = $1",
            claims.document_id,
        )
        .fetch(&mut *conn);

        while let Some(row) = tags.try_next().await? {
            doc.merge_part(row.try_into()?);
        }
    }

    Ok(Json(doc))
}

#[derive(FromRow)]
struct MinutesPerPingRow {
    minutes_per_ping: i32,
    timestamp: DateTime<Utc>,
    counter: i32,
    node: i32,
}

impl TryFrom<MinutesPerPingRow> for Part {
    type Error = Error;

    fn try_from(row: MinutesPerPingRow) -> Result<Part, Self::Error> {
        Ok(Self::MinutesPerPing(Lww::new(
            row.minutes_per_ping.try_into()?,
            Hlc::new_at(row.node.try_into()?, row.timestamp, row.counter.try_into()?),
        )))
    }
}

#[derive(FromRow)]
struct PingRow {
    ping: DateTime<Utc>,
}

impl From<PingRow> for Part {
    fn from(val: PingRow) -> Self {
        Part::Ping(val.ping)
    }
}

#[derive(FromRow)]
struct TagRow {
    ping: DateTime<Utc>,
    tag: Option<String>,
    timestamp: DateTime<Utc>,
    counter: i32,
    node: i32,
}

impl TryFrom<TagRow> for Part {
    type Error = Error;

    fn try_from(row: TagRow) -> Result<Part, Self::Error> {
        Ok(Self::Tag((
            row.ping,
            Lww::new(
                row.tag,
                Hlc::new_at(row.node.try_into()?, row.timestamp, row.counter.try_into()?),
            ),
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handlers::{push, test::TestDoc};
    use beeps_core::NodeId;
    use sqlx::{Pool, Postgres};

    #[test_log::test(sqlx::test)]
    async fn test_pulls_minutes_per_ping(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        document.set_minutes_per_ping(
            document.minutes_per_ping.value() * 2,
            Hlc::new(NodeId::min()),
        );

        let _ = push::handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document.clone()),
        )
        .await
        .unwrap();

        let Json(pulled) = handler(Conn(pool.acquire().await.unwrap()), doc.claims())
            .await
            .unwrap();

        assert_eq!(
            pulled.minutes_per_ping.value(),
            document.minutes_per_ping.value()
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_pulls_pings(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        document.add_ping(Utc::now());

        let _ = push::handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document.clone()),
        )
        .await
        .unwrap();

        let Json(pulled) = handler(Conn(pool.acquire().await.unwrap()), doc.claims())
            .await
            .unwrap();

        assert_eq!(pulled.pings, document.pings);
    }

    #[test_log::test(sqlx::test)]
    async fn test_pulls_tags(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let now = Utc::now();
        document.add_ping(now);
        document.tag_ping(now, "tag".to_string(), Hlc::new(NodeId::min()));

        let _ = push::handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document.clone()),
        )
        .await
        .unwrap();

        let Json(pulled) = handler(Conn(pool.acquire().await.unwrap()), doc.claims())
            .await
            .unwrap();

        assert_eq!(pulled.tags, document.tags);
    }
}
