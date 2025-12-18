use crate::config::AppConfig;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use std::path::Path;
async fn connect_pool(url: &str) -> PgPool {
    match PgPool::connect(url).await {
        Ok(pool) => pool,
        Err(err) => panic!("Failed to create pool: {}", err),
    }
}

async fn run_migrations(pool: &PgPool, migrate_dir: &str) {
    Migrator::new(Path::new(migrate_dir))
        .await
        .expect("Failed to create migrator")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

pub async fn init_db(config: &AppConfig) -> PgPool {
    let url = config.get_database_url();
    let pool = connect_pool(url).await;
    if config.get_run_migrations() {
        run_migrations(&pool, &config.migrate_dir).await;
    };
    return pool;
}
