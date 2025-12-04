use sqlx::PgPool;
use axum::{Router, extract::FromRef};
use crate::router;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

async fn serve() {
    let db = PgPool::connect("").await.unwrap();
    let state = AppState { db };
    let router = Router::new()
        .merge(router::build_router().await)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    
}