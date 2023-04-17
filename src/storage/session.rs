use crate::storage::Db;
use sqlx::any::AnyKind;
use sqlx::AnyPool;
use tracing::info;

impl Db<AnyPool> {
    pub async fn setup_session(&self) -> sqlx::Result<()> {
        let sql = match self.0.connect_options().kind() {
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
        sqlx::query(sql).execute(&self.0).await?;
        Ok(())
    }
}
