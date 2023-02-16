use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::Serialize;
use serde_json::json;

#[allow(dead_code)]
pub fn app_success<T>(data: T) -> ApiSuccess<T> {
    return ApiSuccess { data };
}

#[allow(dead_code)]
pub fn app_error(code: ApiErrorCode) -> ApiError {
    return ApiError {
        code,
        extra: String::from("undefined"),
    };
}

#[allow(dead_code)]
pub fn app_error2(code: ApiErrorCode, extra: String) -> ApiError {
    return ApiError { code, extra };
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
    UserOrPasswordFailed = 3001,
    Unexpected = 9999,
}

pub struct ApiError {
    code: ApiErrorCode,
    extra: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.code as i32,
            "error": self.extra
        }));
        (StatusCode::OK, body).into_response()
    }
}
