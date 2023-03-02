use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::error::Error;

use serde::Serialize;
use serde_json::json;

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

#[allow(dead_code)]
pub enum ApiErrorCode {
    Success = 0,
    // inner error
    InvalidDatabase = 1001,
    // public error
    InvalidSign = 2001,
    InvalidToken = 2002,
    AccountNotFound = 3001,
    AccountOrPasswordFailed = 3002,
    AppNotFound = 4001,
    Unexpected = 9999,
}

pub struct ApiError {
    code: ApiErrorCode,
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.code as i32,
            "error": self.error
        }));
        (StatusCode::OK, body).into_response()
    }
}
