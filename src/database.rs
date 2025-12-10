use sqlx::PgPool;
use crate::config::AppConfig;
pub async fn build_pool() -> PgPool {
    let config = AppConfig::new();
    match PgPool::connect(&config.get_database_url()).await {
        Ok(pool) => pool,
        Err(err) => panic!("Failed to create pool: {}", err),
    }
}