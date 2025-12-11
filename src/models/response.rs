use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(skip)]
    pub code: StatusCode,
    pub message: String,
}
impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> IntoResponse for SuccessResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.code.clone(), Json(self)).into_response()
    }
}
impl ErrorResponse {
    pub fn new(code: StatusCode, message: String) -> Self {
        Self { code, message }
    }
}
