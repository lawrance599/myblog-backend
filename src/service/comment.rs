use crate::models::comment::{
    CommentCreate as ModelCommentCreate, CommentRead, CommentUpdate as ModelCommentUpdate,
};
use crate::repositories::comment::{
    Comment as RepoComment, CommentCreate as RepoCommentCreate, CommentReponsitory,
    CommentUpdate as RepoCommentUpdate,
};
use crate::service::ServiceError;
use sqlx::PgPool;
use tracing::{instrument, event, Level};

pub struct CommentService {
    comment: Box<dyn CommentReponsitory>,
}

impl CommentService {
    pub fn new(pool: PgPool) -> Self {
        tracing::info!("创建CommentService实例成功");
        CommentService {
            comment: Box::new(crate::repositories::comment::SqlxReponsitory::new(pool)),
        }
    }

    #[instrument(
        name = "CommentService::create",
        level = "info",
        skip_all,
        fields(post_id, author)
    )]
    pub async fn create(&self, comment: ModelCommentCreate) -> Result<CommentRead, ServiceError> {
        event!(Level::INFO, post_id = comment.post_id, author = %comment.author, parent_id = ?comment.parent_id, "开始创建评论");

        let comment_create = RepoCommentCreate {
            post_id: comment.post_id,
            author: comment.author.clone(),
            content: comment.content,
            parent_id: comment.parent_id,
        };

        tracing::Span::current().record("post_id", &comment_create.post_id);
        tracing::Span::current().record("author", &comment_create.author);

        let new_comment = self.comment.create(comment_create).await?;
        
        event!(Level::INFO, comment_id = new_comment.id, post_id = new_comment.post_id, author = %new_comment.author, "成功创建评论");
        Ok(convert_repo_comment_to_read(new_comment))
    }

    #[instrument(name = "CommentService::update", level = "info", skip_all, fields(id))]
    pub async fn update(
        &self,
        id: i32,
        comment: ModelCommentUpdate,
    ) -> Result<CommentRead, ServiceError> {
        event!(Level::INFO, comment_id = id, "开始更新评论");

        tracing::Span::current().record("id", &id);

        let comment_update = RepoCommentUpdate {
            id,
            content: comment.content,
        };

        let updated_comment = self.comment.update(comment_update).await?;
        
        event!(Level::INFO, comment_id = id, post_id = updated_comment.post_id, "成功更新评论");
        Ok(convert_repo_comment_to_read(updated_comment))
    }

    #[instrument(name = "CommentService::delete", level = "info", skip_all, fields(id))]
    pub async fn delete(&self, id: i32) -> Result<CommentRead, ServiceError> {
        event!(Level::INFO, comment_id = id, "开始删除评论");

        tracing::Span::current().record("id", &id);

        let deleted_comment = self.comment.delete(id).await?;
        
        event!(Level::INFO, comment_id = id, post_id = deleted_comment.post_id, author = %deleted_comment.author, "成功删除评论");
        Ok(convert_repo_comment_to_read(deleted_comment))
    }

    #[instrument(
        name = "CommentService::find_by_id",
        level = "info",
        skip_all,
        fields(id)
    )]
    pub async fn find_by_id(&self, id: i32) -> Result<CommentRead, ServiceError> {
        event!(Level::INFO, comment_id = id, "开始查询评论");

        tracing::Span::current().record("id", &id);

        let comment = self.comment.find_by_id(id).await?;
        
        event!(Level::INFO, comment_id = id, post_id = comment.post_id, author = %comment.author, "成功查询评论");
        Ok(convert_repo_comment_to_read(comment))
    }

    #[instrument(
        name = "CommentService::find_by_post_id",
        level = "info",
        skip_all,
        fields(post_id)
    )]
    pub async fn find_by_post_id(&self, post_id: i32) -> Result<Vec<CommentRead>, ServiceError> {
        event!(Level::INFO, post_id = post_id, "开始查询文章的所有评论");

        tracing::Span::current().record("post_id", &post_id);

        let comments = self.comment.find_by_post_id(post_id).await?;
        
        event!(Level::INFO, post_id = post_id, comment_count = comments.len(), "成功查询文章的所有评论");
        Ok(comments
            .into_iter()
            .map(convert_repo_comment_to_read)
            .collect())
    }
}

fn convert_repo_comment_to_read(comment: RepoComment) -> CommentRead {
    CommentRead {
        id: comment.id,
        post_id: comment.post_id,
        author: comment.author,
        content: comment.content,
        created_at: comment.created_at.to_string(),
        parent_id: comment.parent_id,
    }
}
