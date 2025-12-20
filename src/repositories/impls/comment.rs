use crate::repositories::{
    ReponsitoryError,
    comment::{Comment, CommentCreate, CommentReponsitory, CommentUpdate},
};
use sqlx::PgPool;
use tracing::{event, Level};
pub struct SqlxReponsitory(pub PgPool);

impl SqlxReponsitory {
    pub fn new(pool: PgPool) -> SqlxReponsitory {
        SqlxReponsitory(pool)
    }
}
#[async_trait::async_trait]
impl CommentReponsitory for SqlxReponsitory {
    async fn create(&self, comment: CommentCreate) -> Result<Comment, ReponsitoryError> {
        event!(Level::DEBUG, post_id = comment.post_id, author = %comment.author, parent_id = ?comment.parent_id, "开始创建评论");
        
        let new: Comment = sqlx::query_as(
            r#"
        INSERT INTO comment(post_id, author, content, parent_id) VALUES($1, $2, $3, $4) RETURNING *"#,
        )
        .bind(comment.post_id)
        .bind(&comment.author)
        .bind(&comment.content)
        .bind(comment.parent_id)
        .fetch_one(&self.0)
        .await?;
        
        event!(Level::DEBUG, comment_id = new.id, post_id = new.post_id, author = %new.author, "成功创建评论");
        Ok(new)
    }
    async fn update(&self, comment: CommentUpdate) -> Result<Comment, ReponsitoryError> {
        event!(Level::DEBUG, comment_id = comment.id, "开始更新评论");
        
        let new: Comment = sqlx::query_as(
            r#"
        UPDATE comment SET content = $1 WHERE id = $2 RETURNING *"#,
        )
        .bind(&comment.content)
        .bind(comment.id)
        .fetch_one(&self.0)
        .await?;
        
        event!(Level::DEBUG, comment_id = new.id, post_id = new.post_id, "成功更新评论");
        Ok(new)
    }
    async fn find_by_id(&self, id: i32) -> Result<Comment, ReponsitoryError> {
        event!(Level::DEBUG, comment_id = id, "开始查询评论");
        
        let comment: Comment = sqlx::query_as(
            r#"
        SELECT * FROM comment WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.0)
        .await?;
        
        event!(Level::DEBUG, comment_id = id, post_id = comment.post_id, author = %comment.author, "成功查询评论");
        Ok(comment)
    }
    async fn delete(&self, id: i32) -> Result<Comment, ReponsitoryError> {
        event!(Level::DEBUG, comment_id = id, "开始删除评论");
        
        let new: Comment = sqlx::query_as(
            r#"
        DELETE FROM comment WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.0)
        .await?;
        
        event!(Level::DEBUG, comment_id = id, post_id = new.post_id, author = %new.author, "成功删除评论");
        Ok(new)
    }
    async fn find_by_post_id(&self, id: i32) -> Result<Vec<Comment>, ReponsitoryError> {
        event!(Level::DEBUG, post_id = id, "开始查询文章的所有评论");
        
        let new: Vec<Comment> = sqlx::query_as(
            r#"
        SELECT * FROM comment WHERE post_id = $1"#,
        )
        .bind(id)
        .fetch_all(&self.0)
        .await?;
        
        event!(Level::DEBUG, post_id = id, comment_count = new.len(), "成功查询文章的所有评论");
        Ok(new)
    }
}
