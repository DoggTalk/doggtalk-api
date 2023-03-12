use axum::{
    async_trait,
    extract::{
        rejection::{JsonRejection, QueryRejection},
        FromRequest, FromRequestParts,
    },
    http::{request::Parts, Request},
};
use serde::de::DeserializeOwned;

use super::error_code::*;
use super::result::*;

pub struct Json<T>(pub T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Json<T>
where
    axum::Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
    S: Send + Sync,
    B: Send + 'static,
{
    type Rejection = ApiError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let req = Request::from_parts(parts, body);

        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(api_error2(
                ApiErrorCode::InvalidParams,
                &parse_error_text(&rejection.body_text()),
            )),
        }
    }
}

pub struct Query<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    axum::extract::Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Query::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(api_error2(
                ApiErrorCode::InvalidParams,
                &parse_error_text(&rejection.body_text()),
            )),
        }
    }
}

fn parse_error_text(text: &str) -> String {
    match text.rfind(": ") {
        Some(pos) => {
            let out = String::from(text.split_at(pos + 2).1);
            match out.find(" at ") {
                Some(pos) => String::from(out.split_at(pos).0),
                _ => out,
            }
        }
        _ => String::from(text),
    }
}
