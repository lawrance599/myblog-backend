use crate::state::AppState;
use axum::Router;
mod post;
pub async fn new() -> Router<AppState> {
    Router::new().nest("/post", post::new().await)
}
