use crate::models::SuccessResponse;
use crate::models::comment::*;
use crate::service::ServiceError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::{
    Json, Router,
    routing::{delete, get, post, put},
};
use tracing::{event, Level};

pub async fn new() -> Router<AppState> {
    Router::new()
        .route("/", post(create_comment))
        .route("/{id}", get(get_comment))
        .route("/{id}", put(update_comment))
        .route("/{id}", delete(delete_comment))
        .route("/post/{post_id}", get(get_comments_by_post_id))
}

/// 创建新评论
pub async fn create_comment(
    State(state): State<AppState>,
    Json(comment): Json<CommentCreate>,
) -> Result<SuccessResponse<CommentRead>, ServiceError> {
    event!(Level::INFO, post_id = comment.post_id, author = %comment.author, "开始创建新评论");
    
    if comment.author.len() > 255 || comment.author.is_empty() {
        event!(Level::WARN, author_length = comment.author.len(), "作者名称长度无效");
        return Err(ServiceError::BadArugment(
            "作者名称长度不能超过255或为空".to_string(),
        ));
    }
    if comment.content.len() > 1000 || comment.content.is_empty() {
        event!(Level::WARN, content_length = comment.content.len(), "评论内容长度无效");
        return Err(ServiceError::BadArugment(
            "评论内容长度不能超过1000或为空".to_string(),
        ));
    }

    let new_comment = state.comment_service.create(comment).await?;
    
    event!(Level::INFO, comment_id = new_comment.id, post_id = new_comment.post_id, author = %new_comment.author, "成功创建新评论");
    Ok(SuccessResponse::new(new_comment))
}

/// 获取单个评论
pub async fn get_comment(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<CommentRead>, ServiceError> {
    event!(Level::INFO, comment_id = id, "开始获取评论");
    
    if id <= 0 {
        event!(Level::WARN, comment_id = id, "无效的评论ID");
        return Err(ServiceError::BadArugment("无效的id".to_string()));
    }

    let comment = state.comment_service.find_by_id(id).await?;
    
    event!(Level::INFO, comment_id = id, post_id = comment.post_id, author = %comment.author, "成功获取评论");
    Ok(SuccessResponse::new(comment))
}

/// 更新评论
pub async fn update_comment(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(comment): Json<CommentUpdate>,
) -> Result<SuccessResponse<CommentRead>, ServiceError> {
    event!(Level::INFO, comment_id = id, "开始更新评论");
    
    if id <= 0 {
        event!(Level::WARN, comment_id = id, "无效的评论ID");
        return Err(ServiceError::BadArugment("无效的id".to_string()));
    }
    if comment.content.len() > 1000 || comment.content.is_empty() {
        event!(Level::WARN, content_length = comment.content.len(), "评论内容长度无效");
        return Err(ServiceError::BadArugment(
            "评论内容长度不能超过1000或为空".to_string(),
        ));
    }

    let updated_comment = state.comment_service.update(id, comment).await?;
    
    event!(Level::INFO, comment_id = id, post_id = updated_comment.post_id, "成功更新评论");
    Ok(SuccessResponse::new(updated_comment))
}

/// 删除评论
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<CommentRead>, ServiceError> {
    event!(Level::INFO, comment_id = id, "开始删除评论");
    
    if id <= 0 {
        event!(Level::WARN, comment_id = id, "无效的评论ID");
        return Err(ServiceError::BadArugment("无效的id".to_string()));
    }

    let deleted_comment = state.comment_service.delete(id).await?;
    
    event!(Level::INFO, comment_id = id, post_id = deleted_comment.post_id, author = %deleted_comment.author, "成功删除评论");
    Ok(SuccessResponse::new(deleted_comment))
}

/// 获取指定文章的所有评论
pub async fn get_comments_by_post_id(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
) -> Result<SuccessResponse<Vec<CommentRead>>, ServiceError> {
    event!(Level::INFO, post_id = post_id, "开始获取文章的所有评论");
    
    if post_id <= 0 {
        event!(Level::WARN, post_id = post_id, "无效的文章ID");
        return Err(ServiceError::BadArugment("无效的post_id".to_string()));
    }

    let comments = state.comment_service.find_by_post_id(post_id).await?;
    
    event!(Level::INFO, post_id = post_id, comment_count = comments.len(), "成功获取文章的所有评论");
    Ok(SuccessResponse::new(comments))
}
