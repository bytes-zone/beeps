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

    pub fn claims(&self) -> Claims {
        Claims::test(self.account_id, self.document_id)
    }
}
