use argon2::password_hash;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to find last insert id")]
    NoLastInsertId,
    #[error("failed to hash the password: {0}")]
    PasswordHashing(password_hash::Error),
}
