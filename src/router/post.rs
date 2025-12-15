use crate::models::post::*;
use crate::models::{ErrorResponse, SuccessResponse};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::{
    Router,
    extract::Multipart,
    http::StatusCode,
    routing::{get, post},
};

pub async fn new() -> Router<AppState> {
    Router::new()
        .route("/upload", post(add_post))
        .route("/{id}/meta", get(read_post_meta))
        .route("/{id}", get(read_post_content))
        .route("/list", get(list_posts))
}
pub async fn list_posts(
    State(state): State<AppState>,
) -> Result<SuccessResponse<Vec<PostMetaRead>>, ErrorResponse> {
    let posts = state.post_service.list_all().await;
    return match posts {
        Ok(posts) => Ok(SuccessResponse::new(
            posts.into_iter().map(|i| i.into()).collect(),
        )),
        Err(err) => Err(ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err)),
    };
}
pub async fn read_post_content(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<PostFull>, ErrorResponse> {
    let post = state
        .post_service
        .read_one(id)
        .await
        .map_err(|e| ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let path = state.post_service.build_file_path(&post.title).await;
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ErrorResponse::new(StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(SuccessResponse::new(PostFull::with_content(post, content)))
}

pub async fn read_post_meta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<SuccessResponse<PostMetaRead>, ErrorResponse> {
    let post = state
        .post_service
        .read_one(id)
        .await
        .map_err(|e| ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(SuccessResponse::new(post.into()))
}

pub async fn add_post(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<SuccessResponse<PostId>, ErrorResponse> {
    let new = process_multipart(multipart)
        .await
        .map_err(|e| ErrorResponse::new(StatusCode::BAD_REQUEST, e))?;
    tracing::Span::current().record("title", &new.title);
    let post = state
        .post_service
        .add_one(new)
        .await
        .map_err(|e| ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(SuccessResponse::new(PostId::new(post.id)))
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
