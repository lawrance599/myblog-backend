mod comment;
mod post;
use crate::models::ErrorResponse;
use crate::repositories::ReponsitoryError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
pub use comment::CommentService;
pub use post::PostService;
use std::io;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("not found")]
    NotFound,
    #[error("bad argument: {0}")]
    BadArugment(String),
    #[error("repository error: {0}")]
    InternalError(String),
    #[error("can't save file")]
    FileError(#[from] io::Error),
}
impl From<ReponsitoryError> for ServiceError {
    fn from(value: ReponsitoryError) -> Self {
        match value {
            ReponsitoryError::NotFound => Self::NotFound,
            _ => Self::InternalError(value.to_string()),
        }
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServiceError::BadArugment(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(StatusCode::BAD_REQUEST, message)),
            )
                .into_response(),
            ServiceError::InternalError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message,
                )),
            )
                .into_response(),
            ServiceError::FileError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message.to_string(),
                )),
            )
                .into_response(),
            ServiceError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    StatusCode::NOT_FOUND,
                    "Not Found".to_string(),
                )),
            )
                .into_response(),
        }
    }
}
