use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{Duration, Utc};
use sqlx::{pool::PoolConnection, query, Acquire, Postgres, Row};

use crate::jwt::Claims;

/// A document for use in testing
pub struct TestDoc {
    pub email: String,
    pub password: String,
    pub document_id: i64,
}

impl TestDoc {
    /// Create a new `TestDoc` for a test
    pub async fn create(pool: &mut PoolConnection<Postgres>) -> Self {
        let email = String::from("test@example.com");
        let password = String::from("letmein");

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("failed to hash password")
            .to_string();

        let mut tx = pool.begin().await.unwrap();

        let account_id: i64 =
            query("INSERT INTO accounts (email, password) VALUES ($1, $2) RETURNING id::BIGINT")
                .bind(&email)
                .bind(&hash)
                .fetch_one(&mut *tx)
                .await
                .expect("failed to insert account")
                .get("id");

        let document_id =
            query("INSERT INTO documents (owner_id) VALUES ($1) RETURNING id::BIGINT")
                .bind(&account_id)
                .fetch_one(&mut *tx)
                .await
                .expect("failed to insert document")
                .get("id");

        tx.commit().await.expect("failed to commit transaction");

        TestDoc {
            email,
            password,
            document_id,
        }
    }

    /// Get appropriate claims for this doc
    pub fn claims(&self) -> Claims {
        Claims {
            sub: self.email.clone(),
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + Duration::days(90)).timestamp(),
        }
    }
}
