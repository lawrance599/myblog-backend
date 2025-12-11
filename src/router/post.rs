use crate::models::post::*;
use crate::models::{Error, Success};
use crate::serve::AppState;
use axum::extract::State;
use axum::{Router, extract::Multipart, http::StatusCode, routing::post};

pub async fn new() -> Router<AppState> {
    Router::new().route("/upload", post(add_post))
}
#[tracing::instrument(level = "info", skip_all, name = "upload post", fields(title))]
pub async fn add_post(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Success<PostId>, Error> {
    let new = process_multipart(multipart)
        .await
        .map_err(|e| Error::new(StatusCode::BAD_REQUEST, e))?;
    tracing::Span::current().record("title", &new.title);
    let post = state
        .post_service
        .add_post(new)
        .await
        .map_err(|e| Error::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Success::new(PostId::new(post.id)))
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
