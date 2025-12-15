mod impls;
pub mod post;
#[derive(Debug, thiserror::Error)]
pub enum ReponsitoryError {
    #[error("Not Found")]
    NotFound,
    #[error("Pool Error: {0}")]
    PoolError(String),
    #[error("Database Error: {0}")]
    DataBaseError(String),
    #[error("Internal Error")]
    InternalError,
}
impl From<sqlx::Error> for ReponsitoryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => ReponsitoryError::NotFound,
            sqlx::Error::PoolTimedOut => ReponsitoryError::PoolError("Timed Out".to_string()),
            sqlx::Error::PoolClosed => ReponsitoryError::PoolError("Closed".to_string()),
            sqlx::Error::Database(e) => ReponsitoryError::DataBaseError(e.message().to_string()),
            sqlx::Error::Tls(e) => ReponsitoryError::DataBaseError(e.to_string()),
            sqlx::Error::Io(e) => ReponsitoryError::DataBaseError(e.to_string()),
            _ => ReponsitoryError::InternalError,
        }
    }
}
