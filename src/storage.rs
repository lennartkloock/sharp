use sqlx::{any::AnyPoolOptions, AnyPool};

use crate::storage::error::StorageResult;
pub use session::*;
pub use user::*;

pub mod error;
pub mod session;
pub mod user;

pub struct DbPool(AnyPool);

impl DbPool {
    pub async fn connect(url: &str) -> sqlx::Result<Self> {
        Ok(Self(AnyPoolOptions::new().connect(url).await?))
    }

    pub async fn setup(&self) -> StorageResult<()> {
        user::setup(self).await?;
        session::setup(self).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::storage::{DbPool, NewUser};

    #[tokio::test]
    async fn fetch_test() {
        let pool = DbPool::connect("sqlite::memory:").await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&pool.0)
            .await
            .unwrap();
        assert_eq!(row.0, 150);
    }

    #[tokio::test]
    async fn store_test() {
        let pool = DbPool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE people (name varchar(255))")
            .execute(&pool.0)
            .await
            .unwrap();
        sqlx::query("INSERT INTO people (name) VALUES ($1)")
            .bind("Test")
            .execute(&pool.0)
            .await
            .unwrap();
        let row: (String,) = sqlx::query_as("SELECT name FROM people")
            .fetch_one(&pool.0)
            .await
            .unwrap();
        assert_eq!(row.0, "Test");
    }

    #[tokio::test]
    async fn insert_user() {
        let pool = DbPool::connect("sqlite:sharp-test.sqlite").await.unwrap();
        println!(
            "User id: {}",
            pool.insert_user(NewUser {
                email: "USER".to_string(),
                username: None,
                password: "TESTPASS".to_string()
            })
            .await
            .unwrap()
        );
    }
}
