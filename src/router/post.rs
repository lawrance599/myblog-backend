use axum::{
    extract::{Multipart},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use crate::config::AppConfig;
async fn build_router() -> Router<AppConfig> {
    todo!()
}

