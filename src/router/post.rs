use crate::models::Pagenigation;
use crate::models::SuccessResponse;
use crate::models::post::*;
use crate::service::ServiceError;
use crate::state::AppState;
use axum::extract::Query;
use axum::extract::{Path, State};
use axum::{
    Router,
    extract::Multipart,
    routing::{get, post},
};

pub async fn new() -> Router<AppState> {
    Router::new()
        .route("/upload", post(add_post))
        .route("/{id}/meta", get(read_post_meta))
        .route("/{id}", get(read_post_content))
        .route("/list", get(list_posts))
}

/// 分页访问报告的元数据
///
/// # Arguments
///
/// - `Query(pagenigation` (`undefined`) - 分页参数.
///
pub async fn list_posts(
    Query(pagenigation): Query<Pagenigation>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<Vec<PostMetaRead>>, ServiceError> {
    let posts = state.post_service.list(pagenigation).await?;
    Ok(SuccessResponse::new(
        posts.into_iter().map(|p| p.into()).collect(),
    ))
}
pub async fn read_post_content(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<Post>, ServiceError> {
    let post = state.post_service.read_one(id).await?;
    let path = state.post_service.build_file_path(&post.title).await;
    let content = tokio::fs::read_to_string(path).await?;
    Ok(SuccessResponse::new(Post::with_content(post, content)))
}

pub async fn read_post_meta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<PostMetaRead>, ServiceError> {
    let post = state.post_service.read_one(id).await?;
    Ok(SuccessResponse::new(post.into()))
}

pub async fn add_post(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<SuccessResponse<PostMetaRead>, ServiceError> {
    let new = match process_multipart(multipart).await {
        Ok(new) => Ok(new),
        Err(e) => Err(ServiceError::BadArugment(e)),
    }?;

    if new.title.len() > 255 || new.title.is_empty() {
        return Err(ServiceError::BadArugment(
            "标题长度不能超过255或为空".to_string(),
        ));
    }
    if new.tags.len() > 10 {
        return Err(ServiceError::BadArugment("标签长度不能超过10".to_string()));
    }
    if new.content.len() > 1024 * 1024 * 10 {
        return Err(ServiceError::BadArugment("内容长度不能超过10M".to_string()));
    }

    tracing::Span::current().record("title", &new.title);
    let post = state.post_service.add_one(new).await?;

    Ok(SuccessResponse::new(post.into()))
}
#[inline]
async fn process_multipart(mut multipart: Multipart) -> Result<PostCreate, String> {
    let mut post = PostCreate::default();
    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        match field.name() {
            None => continue,
            Some("title") => {
                post.title = field
                    .text()
                    .await
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
            }
            Some("tags") => {
                post.tags = field
                    .text()
                    .await
                    .map_err(|e| e.to_string())?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                post.tags.dedup();
            }
            Some("content") => {
                post.content = field.bytes().await.map_err(|e| e.to_string())?.into()
            }
            Some(_) => return Err("无效的字段".to_string()),
        }
    }
    Ok(post)
}
