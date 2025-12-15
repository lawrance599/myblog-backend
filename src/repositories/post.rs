use super::ReponsitoryError;
use async_trait::async_trait;
use sqlx::FromRow;
use sqlx::types::Json;
use sqlx::types::chrono::{DateTime, Utc};
/// 存储的博文结构
#[allow(dead_code)]
#[derive(FromRow)]
pub struct PostMeta {
    pub id: i32,
    pub title: String,
    pub tags: Json<Vec<String>>,
    pub first_publish: DateTime<Utc>,
    pub last_modify: DateTime<Utc>,
    pub count: i32,
}

pub struct PostMetaCreate {
    pub title: String,
    pub tags: Vec<String>,
    pub kw: Vec<String>,
}

pub struct PostMetaUpdate {
    pub id: i32,
    pub title: String,
    pub tags: Vec<String>,
    pub kw: Vec<String>,
}
#[async_trait]
pub trait PostMetaReponsitory: Send + Sync {
    async fn list_all(&self) -> Result<Vec<PostMeta>, ReponsitoryError>;
    async fn find_by_id(&self, id: i32) -> Result<PostMeta, ReponsitoryError>;
    async fn find_by_keywords(
        &self,
        keywords: &[String],
    ) -> Result<Vec<PostMeta>, ReponsitoryError>;
    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<PostMeta>, ReponsitoryError>;
    async fn add(&self, post: PostMetaCreate) -> Result<PostMeta, ReponsitoryError>;
    async fn update(&self, post: PostMetaUpdate) -> Result<PostMeta, ReponsitoryError>;
    async fn delete(&self, id: i32) -> Result<(), ReponsitoryError>;
}
pub use super::impls::post::SqlxReponsitory;
