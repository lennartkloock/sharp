use crate::storage::{
    error::{StorageError, StorageResult},
    Db,
};
use argon2::{
    password_hash,
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use sqlx::{any::AnyKind, Any, Executor};
use tracing::info;

pub type UserId = i64;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    // pub created_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct NewUser {
    pub email: String,
    pub username: Option<String>,
    pub password: String,
}

pub async fn setup(db: &Db) -> StorageResult<()> {
    let sql = match db.connect_options().kind() {
        AnyKind::Sqlite => {
            "CREATE TABLE users
(
    id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    email         TEXT NOT NULL,
    username      TEXT,
    password_hash TEXT NOT NULL,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);
"
        }
        _ => {
            "CREATE TABLE users
(
    id            INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    email         TEXT NOT NULL,
    username      TEXT,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"
        }
    };
    info!("creating `users` table");
    sqlx::query(sql).execute(db).await?;
    Ok(())
}

pub async fn insert<'a, E: Executor<'a, Database = Any>>(
    e: E,
    new_user: &NewUser,
) -> StorageResult<UserId> {
    let pass_hash = hash_password(&new_user.password).map_err(StorageError::PasswordHashing)?;
    let res = sqlx::query("INSERT INTO users (email, username, password_hash) VALUES (?, ?, ?)")
        .bind(&new_user.email.to_lowercase())
        .bind(&new_user.username)
        .bind(pass_hash)
        .execute(e)
        .await?;
    let id = res.last_insert_id().ok_or(StorageError::NoLastInsertId)?;
    info!("created new user with id {id}");
    Ok(id)
}

pub async fn get<'a, E: Executor<'a, Database = Any>>(
    e: E,
    email: &str,
) -> StorageResult<Option<User>> {
    Ok(
        sqlx::query_as("SELECT id, email, username, password_hash FROM users WHERE email = ?")
            .bind(email.to_lowercase())
            .fetch_optional(e)
            .await?,
    )
}

fn hash_password(password: &str) -> password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
}