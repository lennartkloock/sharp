use crate::storage::DbPool;
use sqlx::any::AnyKind;
use tracing::info;

pub async fn setup(db: &DbPool) -> sqlx::Result<()> {
    let sql = match db.0.connect_options().kind() {
        AnyKind::Sqlite => {
            "CREATE TABLE sessions
(
    id      INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token   INTEGER NOT NULL
);
"
        }
        _ => {
            "CREATE TABLE sessions
(
    id      INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    token   INTEGER NOT NULL
);
"
        }
    };
    info!("creating `sessions` table");
    sqlx::query(sql).execute(&db.0).await.map(|_| ())
}
