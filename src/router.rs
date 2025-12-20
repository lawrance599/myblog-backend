use crate::state::AppState;
use axum::Router;
mod post;
mod comment;
pub async fn new() -> Router<AppState> {
    Router::new()
        .nest("/post", post::new().await)
        .nest("/comment", comment::new().await)
}
