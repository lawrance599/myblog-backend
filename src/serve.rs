use std::ops::Deref;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::database::init_db;
use crate::router;
use crate::service::post::PostService;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::prelude::*;

pub struct Inner {
    pub config: AppConfig,
    pub post_service: PostService,
}
impl Inner {
    pub async fn new(config: AppConfig) -> Self {
        let url = config.get_database_url();
        info!("使用`{}`连接数据库", url);
        let pool = init_db(&config).await;
        let post_service = PostService::new(pool.clone(), config.get_save_dir());
        Inner {
            config,
            post_service,
        }
    }
}
#[derive(Clone)]
pub struct AppState(Arc<Inner>);
impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        AppState(Arc::new(Inner::new(config).await))
    }
}
impl Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

pub async fn serve() {
    let config = AppConfig::new();
    let addr = config.get_listener_addr();
    let log_level = config.get_log_level();

    // TODO: 将layer设置分离到到单独的配置文件中
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .pretty()
                .with_target(true),
        )
        .with(
            tracing_subscriber::EnvFilter::try_new(config.get_log_level())
                .unwrap_or(tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();
    tracing::info!("应用日志记录等级为: {}", log_level);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(item) => item,
        Err(err) => panic!("{}", err),
    };
    tracing::info!("应用正在监听: {}", &addr);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let state = AppState::new(config).await;
    let router = Router::new()
        .merge(router::new().await)
        .layer(cors)
        .with_state(state);

    axum::serve(listener, router).await.unwrap();
}
