use axum::{Router, routing::get};
use super::serve::AppState;
mod post;
pub async fn build_router() -> Router<AppState> {
    todo!()
}