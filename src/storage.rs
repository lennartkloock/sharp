use crate::storage::error::StorageResult;
use sqlx::AnyPool;

pub mod error;
pub mod session;
pub mod user;

pub type Db = AnyPool;

pub async fn setup(db: &Db) -> StorageResult<()> {
    user::setup(db).await?;
    session::setup(db).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::storage::{user, user::NewUser, Db};
    use std::env;

    #[tokio::test]
    async fn fetch_test() {
        let pool = Db::connect("sqlite::memory:").await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 150);
    }

    #[tokio::test]
    async fn store_test() {
        let pool = Db::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE people (name varchar(255))")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO people (name) VALUES ($1)")
            .bind("Test")
            .execute(&pool)
            .await
            .unwrap();
        let row: (String,) = sqlx::query_as("SELECT name FROM people")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, "Test");
    }

    #[tokio::test]
    async fn insert_user() {
        let is_ci = env::var("CI").map(|v| v == "true").unwrap_or(false);
        let url = if is_ci {
            "sqlite::memory:"
        } else {
            "sqlite:sharp-test.sqlite"
        };
        let pool = Db::connect(url).await.unwrap();
        println!(
            "User id: {}",
            user::insert(
                &pool,
                &NewUser {
                    email: "USER".to_string(),
                    username: Some("USERNAME".to_string()),
                    password: "TESTPASS".to_string(),
                }
            )
            .await
            .unwrap()
        );
    }
}
