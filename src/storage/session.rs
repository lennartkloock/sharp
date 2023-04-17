use crate::storage::{
    error::{StorageError, StorageResult},
    user::UserId,
    Db,
};
use base64::Engine;
use rand::Rng;
use sqlx::{any::AnyKind, Any, Executor};
use time::Duration;
use tracing::info;

pub type SessionId = i64;

pub const MAX_AGE: Duration = Duration::days(1);

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub token: String,
    pub created_at: time::OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct NewSession {
    pub user_id: UserId,
    pub token: String,
}

impl NewSession {
    pub fn generate(user_id: UserId) -> Self {
        let token: [u8; 16] = rand::thread_rng().gen();
        Self {
            user_id,
            token: base64::engine::general_purpose::STANDARD.encode(&token),
        }
    }
}

pub async fn setup(db: &Db) -> sqlx::Result<()> {
    let sql = match db.connect_options().kind() {
        AnyKind::Sqlite => {
            "CREATE TABLE sessions
(
    id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL,
    token      TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"
        }
        _ => {
            "CREATE TABLE sessions
(
    id         INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id    INTEGER NOT NULL,
    token      TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"
        }
    };
    info!("creating `sessions` table");
    sqlx::query(sql).execute(db).await?;
    Ok(())
}

pub async fn insert<'a, E: Executor<'a, Database = Any>>(
    e: E,
    new_session: &NewSession,
) -> StorageResult<SessionId> {
    let res = sqlx::query("INSERT INTO sessions (user_id, token) VALUES (?, ?)")
        .bind(&new_session.user_id)
        .bind(&new_session.token)
        .execute(e)
        .await?;
    let id = res.last_insert_id().ok_or(StorageError::NoLastInsertId)?;
    info!("created new session with id {id}");
    Ok(id)
}
