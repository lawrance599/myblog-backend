use super::ReponsitoryError;
pub use super::impls::comment::SqlxReponsitory;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
#[derive(Deserialize, Serialize)]
pub struct CommentCreate {
    pub post_id: i32,
    pub author: String,
    pub content: String,
    pub parent_id: Option<i32>,
}
#[derive(Debug, Serialize)]
pub struct CommentUpdate {
    pub id: i32,
    pub content: String,
}
#[derive(FromRow)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<i32>,
}

#[async_trait]
pub trait CommentReponsitory: Send + Sync {
    async fn create(&self, comment: CommentCreate) -> Result<Comment, ReponsitoryError>;
    async fn update(&self, comment: CommentUpdate) -> Result<Comment, ReponsitoryError>;
    async fn delete(&self, id: i32) -> Result<Comment, ReponsitoryError>;
    async fn find_by_id(&self, id: i32) -> Result<Comment, ReponsitoryError>;
    async fn find_by_post_id(&self, id: i32) -> Result<Vec<Comment>, ReponsitoryError>;
}
