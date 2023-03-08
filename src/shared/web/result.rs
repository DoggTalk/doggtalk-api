use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::error::Error;

use super::error_code::*;

#[allow(dead_code)]
pub fn api_success<T>(data: T) -> ApiSuccess<T> {
    return ApiSuccess { data };
}

#[allow(dead_code)]
pub fn api_error(code: ApiErrorCode) -> ApiError {
    return ApiError {
        code,
        error: String::from("undefined"),
    };
}

#[allow(dead_code)]
pub fn api_error2(code: ApiErrorCode, extra: &str) -> ApiError {
    return ApiError {
        code,
        error: String::from(extra),
    };
}

#[allow(dead_code)]
pub fn api_errore(code: ApiErrorCode, e: &dyn Error) -> ApiError {
    return ApiError {
        code,
        error: String::from(e.to_string()),
    };
}

pub struct ApiSuccess<T> {
    data: T,
}

impl<T> IntoResponse for ApiSuccess<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": ApiErrorCode::Success as i32,
            "error": "ok",
            "data": self.data,
        }));
        (StatusCode::OK, body).into_response()
    }
}

pub struct ApiError {
    code: ApiErrorCode,
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (code, error) = render_error(self.code, &self.error);
        let body = Json(json!({
            "code": code,
            "error": error
        }));
        (StatusCode::OK, body).into_response()
    }
}
