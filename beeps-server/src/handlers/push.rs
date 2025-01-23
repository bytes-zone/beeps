use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::http::StatusCode;
use axum::Json;
use beeps_core::document::Part;
use beeps_core::merge::Merge;
use beeps_core::sync::push;
use sqlx::{Acquire, QueryBuilder};

#[tracing::instrument]
#[expect(clippy::cast_lossless)]
pub async fn handler(
    Conn(mut conn): Conn,
    claims: Claims,
    Json(req): Json<push::Req>,
) -> Result<Json<push::Resp>, Error> {
    // Validate that the user owns the document
    let authed_document = sqlx::query!(
        "SELECT documents.id FROM documents \
        JOIN accounts ON accounts.id = documents.owner_id \
        WHERE accounts.email = $1 AND documents.id = $2",
        claims.sub,
        req.document_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

    bail_if!(
        authed_document.is_none(),
        "Document not found",
        StatusCode::NOT_FOUND
    );

    let mut minutes_per_pings = vec![];
    let mut pings = vec![];
    let mut tags = vec![];

    req.document.split().for_each(|item| match item {
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
            "INSERT INTO minutes_per_pings (document_id, minutes_per_ping, clock, counter, node_id)",
        );
        query.push_values(minutes_per_pings, |mut b, value| {
            let clock = value.clock();

            let value = *value.value() as i32;
            let counter: i64 = clock
                .counter()
                .try_into()
                .expect("counter should fit in i64");
            let node = clock.node().0 as i32;

            b.push_bind(req.document_id)
                .push_bind(value)
                .push_bind(clock.timestamp())
                .push_bind(counter)
                .push_bind(node);
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    if !pings.is_empty() {
        let mut query = QueryBuilder::new("INSERT INTO pings (document_id, ping)");
        query.push_values(pings, |mut b, value| {
            b.push_bind(req.document_id).push_bind(value);
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    if !tags.is_empty() {
        let mut query =
            QueryBuilder::new("INSERT INTO tags (document_id, ping, tag, clock, counter, node_id)");
        query.push_values(tags, |mut b, (ping, tag)| {
            let clock = tag.clock();

            let counter: i64 = clock
                .counter()
                .try_into()
                .expect("counter should fit in i64");
            let node = clock.node().0 as i32;

            b.push_bind(req.document_id)
                .push_bind(ping)
                .push_bind(tag.value().clone())
                .push_bind(clock.timestamp())
                .push_bind(counter)
                .push_bind(node);
        });
        query.push("ON CONFLICT DO NOTHING");
        query.build().execute(&mut *tx).await?;
    }

    tx.commit().await?;

    Ok(Json(push::Resp {}))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handlers::test::TestDoc;
    use beeps_core::{Document, Hlc, NodeId};
    use chrono::Utc;
    use sqlx::{pool::PoolConnection, query, Pool, Postgres, Row};

    #[test_log::test(sqlx::test)]
    fn test_unknown_document_not_authorized(mut conn: PoolConnection<Postgres>) {
        let doc = TestDoc::create(&mut conn).await;

        let err = handler(
            Conn(conn),
            doc.claims(),
            Json(push::Req {
                document_id: doc.document_id + 1,
                document: Document::default(),
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(
            err.unwrap_custom(),
            (StatusCode::NOT_FOUND, "Document not found".to_string())
        )
    }

    #[test_log::test(sqlx::test)]
    fn test_inserts_minutes_per_ping(pool: Pool<Postgres>) {
        let doc = TestDoc::create(&mut pool.acquire().await.unwrap()).await;

        let mut document = Document::default();
        let clock = Hlc::new(NodeId::min());
        document.set_minutes_per_ping(60, clock);

        let _ = handler(
            Conn(pool.acquire().await.unwrap()),
            doc.claims(),
            Json(push::Req {
                document_id: doc.document_id,
                document,
            }),
        )
        .await
        .unwrap();

        let inserted = query!(
            "SELECT minutes_per_ping, clock, counter, node_id FROM minutes_per_pings WHERE document_id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert_eq!(inserted.minutes_per_ping, 60);
        assert_eq!(inserted.clock, clock.timestamp());
        assert_eq!(inserted.counter as u64, clock.counter());
        assert_eq!(inserted.node_id as u16, clock.node().0);
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
            Json(push::Req {
                document_id: doc.document_id,
                document,
            }),
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

        assert_eq!(inserted.ping, now);
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
            Json(push::Req {
                document_id: doc.document_id,
                document,
            }),
        )
        .await
        .unwrap();

        let inserted = query!(
            "SELECT ping, tag, clock, counter, node_id FROM tags WHERE document_id = $1",
            doc.document_id
        )
        .fetch_one(&mut *pool.acquire().await.unwrap())
        .await
        .unwrap();

        assert_eq!(inserted.ping, now);
        assert_eq!(inserted.tag, "test".to_string());
        assert_eq!(inserted.clock, clock.timestamp());
        assert_eq!(inserted.counter as u64, clock.counter());
        assert_eq!(inserted.node_id as u16, clock.node().0);
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
            Json(push::Req {
                document_id: doc.document_id,
                document: document.clone(),
            }),
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
            Json(push::Req {
                document_id: doc.document_id,
                document,
            }),
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
}
