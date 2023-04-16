use crate::storage::{
    error::{StorageError, StorageResult},
    DbPool,
};
use sqlx::any::AnyKind;
use tracing::info;

type UserId = i64;

pub async fn setup(db: &DbPool) -> sqlx::Result<()> {
    let sql = match db.0.connect_options().kind() {
        AnyKind::Sqlite => {
            "CREATE TABLE users
(
    id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    email         TEXT NOT NULL,
    username      TEXT,
    password_hash TEXT NOT NULL
);
"
        }
        _ => {
            "CREATE TABLE users
(
    id            INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    email         TEXT NOT NULL,
    username      TEXT,
    password_hash TEXT NOT NULL
);
"
        }
    };
    info!("creating `users` table");
    sqlx::query(sql).execute(&db.0).await.map(|_| ())
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct NewUser {
    pub email: String,
    pub username: Option<String>,
    pub password: String,
}

impl DbPool {
    pub async fn insert_user(&self, new_user: NewUser) -> StorageResult<UserId> {
        let res = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
            .bind(new_user.email)
            .bind(new_user.password)
            .execute(&self.0)
            .await?;
        res.last_insert_id().ok_or(StorageError::NoLastInsertId)
    }
}
