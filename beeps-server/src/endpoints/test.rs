use chrono::{DateTime, TimeZone};
use common::hlc::Hlc;
use common::op::Op;
use sqlx::{pool::PoolConnection, query, Acquire, Postgres};

use crate::auth::Claims;

pub struct Doc {
    pub account_id: i64,
    pub document_id: i64,
}

impl Doc {
    pub async fn create(pool: &mut PoolConnection<Postgres>) -> Self {
        let mut tx = pool.begin().await.unwrap();

        let account = query!("INSERT INTO accounts DEFAULT VALUES RETURNING id::BIGINT")
            .fetch_one(&mut *tx)
            .await
            .unwrap();

        let document = query!(
            "INSERT INTO documents (account_id) VALUES ($1) RETURNING id::BIGINT",
            account.id.unwrap()
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        tx.commit().await.unwrap();

        Doc {
            account_id: account.id.unwrap(),
            document_id: document.id.unwrap(),
        }
    }

    pub async fn add_device(&self, conn: &mut PoolConnection<Postgres>, name: &str) -> i64 {
        query!(
            "INSERT INTO devices (document_id, name) VALUES ($1, $2) RETURNING id::BIGINT",
            self.document_id,
            name
        )
        .fetch_one(&mut **conn)
        .await
        .unwrap()
        .id
        .unwrap()
    }

    pub async fn add_op(&self, conn: &mut PoolConnection<Postgres>, clock: &Hlc, op: &Op) -> i64 {
        let result = query!(
            r#"
                INSERT INTO operations (document_id, timestamp, counter, op, device_id)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id::BIGINT
            "#,
            self.document_id,
            clock.timestamp,
            clock.counter,
            serde_json::to_value(op).unwrap(),
            clock.node
        )
        .fetch_one(&mut **conn)
        .await
        .unwrap();

        result.id.unwrap()
    }

    pub fn claims(&self) -> Claims {
        Claims::test(self.account_id, self.document_id)
    }
}

pub fn trunc_ms<Z: TimeZone>(input: DateTime<Z>) -> DateTime<Z> {
    DateTime::from_timestamp_micros(input.timestamp_micros())
        .unwrap()
        .with_timezone(&input.timezone())
}
