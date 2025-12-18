use crate::config::AppConfig;
use crate::database::init_db;
use crate::service::PostService;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;
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

        info!("初始化分词器");
        Inner {
            config,
            post_service: post_service,
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
        &self.0
    }
}
