use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Success<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct Error {
    #[serde(skip)]
    pub code: StatusCode,
    pub message: String,
}
impl<T> Success<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}
impl<T> IntoResponse for Success<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.code.clone(), Json(self)).into_response()
    }
}
impl Error {
    pub fn new(code: StatusCode, message: String) -> Self {
        Self { code, message }
    }
}
