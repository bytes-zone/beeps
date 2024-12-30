use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use sqlx::{pool::PoolConnection, query, Acquire, Postgres};

/// A document for use in testing
pub struct TestDoc {
    pub email: String,
    pub password: String,
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

        query("INSERT INTO accounts (email, password) VALUES ($1, $2) RETURNING id::BIGINT")
            .bind(&email)
            .bind(&hash)
            .fetch_one(&mut *tx)
            .await
            .expect("failed to insert account");

        tx.commit().await.expect("failed to commit transaction");

        TestDoc { email, password }
    }
}
