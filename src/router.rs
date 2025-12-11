use crate::serve::AppState;
use axum::Router;
mod post;
pub async fn new() -> Router<AppState> {
    Router::new().nest("/post", post::new().await)
}
