use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::document::Part;
use beeps_core::merge::Merge;
use beeps_core::sync::push;
use sqlx::{query, Acquire, QueryBuilder};

#[tracing::instrument]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Json(req): Json<push::Req>,
) -> Result<Json<push::Resp>, Error> {
    let mut minutes_per_pings = vec![];
    let mut pings = vec![];
    let mut tags = vec![];

    req.split().for_each(|item| match item {
        Part::MinutesPerPing(minutes) => {
            minutes_per_pings.push(minutes);
        }
        Part::Ping(ping) => {
            pings.push(ping);
        }
        Part::Tag((ping, tag)) => {
            tags.push((ping, tag));
        }
    });

    let mut tx = conn.begin().await?;

    if !minutes_per_pings.is_empty() {
        let mut query = QueryBuilder::new(
            "INSERT INTO minutes_per_pings (document_id, minutes_per_ping, timestamp, counter, node)",
        );
        query.push_values(minutes_per_pings, |mut b, value| {
            let clock = value.clock();

            b.push_bind(claims.document_id)
                .push_bind(i32::from(*value.value()))
                .push_bind(clock.timestamp())
                .push_bind(i32::from(clock.counter()))
                .push_bind(i32::from(*clock.node()));
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    if !pings.is_empty() {
        let mut query = QueryBuilder::new("INSERT INTO pings (document_id, ping)");
        query.push_values(pings, |mut b, value| {
            b.push_bind(claims.document_id).push_bind(value);
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    if !tags.is_empty() {
        let mut query = QueryBuilder::new(
            "INSERT INTO tags (document_id, ping, tag, timestamp, counter, node)",
        );
        query.push_values(tags, |mut b, (ping, tag)| {
            let clock = tag.clock();

            b.push_bind(claims.document_id)
                .push_bind(ping)
                .push_bind(tag.value().clone())
                .push_bind(clock.timestamp())
                .push_bind(i32::from(clock.counter()))
                .push_bind(i32::from(*clock.node()));
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    query!(
        "UPDATE documents SET updated_at = NOW() WHERE id = $1",
        claims.document_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(push::Resp {}))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{assert_eq_timestamps, handlers::test::TestDoc};
    use beeps_core::{Document, Hlc, NodeId};
    use chrono::Utc;
    use sqlx::{Pool, Postgres, Row};

    #[test_log::test(sqlx::test)]
    fn test_inserts_minutes_per_ping(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let clock = Hlc::new(NodeId::min());
        document.set_minutes_per_ping(60, clock);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document),
        )
        .await
        .unwrap();

        let inserted = query!(
            "SELECT minutes_per_ping, timestamp, counter, node FROM minutes_per_pings WHERE document_id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert_eq!(inserted.minutes_per_ping, 60);
        assert_eq_timestamps!(inserted.timestamp, clock.timestamp());
        assert_eq!(inserted.counter, i32::from(clock.counter()));
        assert_eq!(inserted.node, i32::from(*clock.node()));
    }

    #[test_log::test(sqlx::test)]
    fn test_inserts_pings(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let now = Utc::now();
        document.add_ping(now);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document),
        )
        .await
        .unwrap();

        let inserted = query!(
            "SELECT ping FROM pings WHERE document_id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert_eq_timestamps!(inserted.ping, now);
    }

    #[test_log::test(sqlx::test)]
    fn test_inserts_tags(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let now = Utc::now();
        let clock = Hlc::new(NodeId::min());
        document.add_ping(now);
        document.tag_ping(now, "test".to_string(), clock);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document),
        )
        .await
        .unwrap();

        let inserted = query!(
            "SELECT ping, tag, timestamp, counter, node FROM tags WHERE document_id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert_eq_timestamps!(inserted.ping, now);
        assert_eq!(inserted.tag, Some("test".to_string()));
        assert_eq_timestamps!(inserted.timestamp, clock.timestamp());
        assert_eq!(inserted.counter, i32::from(clock.counter()));
        assert_eq!(inserted.node, i32::from(*clock.node()));
    }

    macro_rules! table_size {
        ($table:expr, $document_id:expr, $pool:expr) => {
            query(&format!(
                "SELECT COUNT(*) FROM {} WHERE document_id = $1",
                $table
            ))
            .bind($document_id)
            .fetch_one(&mut *$pool.acquire().await.unwrap())
            .await
            .unwrap()
            .get("count")
        };
    }

    #[test_log::test(sqlx::test)]
    fn test_idempotent(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let now = Utc::now();
        let clock = Hlc::new(NodeId::min());
        document.set_minutes_per_ping(60, clock);
        document.add_ping(now);
        document.tag_ping(now, "test".to_string(), clock);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document.clone()),
        )
        .await
        .unwrap();

        let num_minutes_per_ping_before: i64 =
            table_size!("minutes_per_pings", doc.document_id, pool);
        let num_pings_before: i64 = table_size!("pings", doc.document_id, pool);
        let num_tags_before: i64 = table_size!("tags", doc.document_id, pool);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(document),
        )
        .await
        .unwrap();

        let num_minutes_per_ping_after: i64 =
            table_size!("minutes_per_pings", doc.document_id, pool);
        let num_pings_after: i64 = table_size!("pings", doc.document_id, pool);
        let num_tags_after: i64 = table_size!("tags", doc.document_id, pool);

        assert_eq!(num_minutes_per_ping_before, num_minutes_per_ping_after);
        assert_eq!(num_pings_before, num_pings_after);
        assert_eq!(num_tags_before, num_tags_after);
    }

    #[test_log::test(sqlx::test)]
    fn test_updates_updated_at(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let before = query!(
            "SELECT updated_at FROM documents WHERE id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(Document::default()),
        )
        .await
        .unwrap();

        let after = query!(
            "SELECT updated_at FROM documents WHERE id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert!(
            before.updated_at < after.updated_at,
            "updated_at was not updated: {} (before) is not greater than {} (after)",
            before.updated_at,
            after.updated_at
        );
    }
}
