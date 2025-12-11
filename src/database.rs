use sqlx::PgPool;

use crate::config::AppConfig;
async fn connect_pool(url: &str) -> PgPool {
    match PgPool::connect(url).await {
        Ok(pool) => pool,
        Err(err) => panic!("Failed to create pool: {}", err),
    }
}

async fn run_migrations() {
    todo!()
}

pub async fn init_db(config: &AppConfig) -> PgPool {
    let url = config.get_database_url();
    let pool = connect_pool(url).await;
    if config.get_run_migrations() {
        run_migrations().await;
    };
    return pool;
}
