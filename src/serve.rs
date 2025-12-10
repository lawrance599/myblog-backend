use sqlx::PgPool;
use axum::{Router};
use crate::config::AppConfig;
use crate::router;
use crate::database::build_pool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub async fn serve() {
    let config = AppConfig::new();
    let pool = build_pool().await;
    let state = AppState { pool };
    let router = Router::new()
        .merge(router::build_router().await)
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(config.get_listener_addr()).await {
        Ok(item) => item,
        Err(err) => panic!("{}", err)
    };
    axum::serve(listener, router).await.unwrap();
}