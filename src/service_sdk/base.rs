use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};

use crate::shared::jwt::*;
use crate::shared::web::*;

const SDK_TC: &'static str = "sdk";

pub struct UserClaims {
    pub app_id: u64,
    pub user_id: u64,
}

impl Default for UserClaims {
    fn default() -> Self {
        Self {
            app_id: 0,
            user_id: 0,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserClaims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| api_error2(ApiErrorCode::InvalidToken, "empty"))?;

        let res = jwt_parse(SDK_TC, bearer.token())?;
        let parts: Vec<&str> = res.split("@").collect();

        if parts.len() != 2 {
            return Err(api_error2(ApiErrorCode::InvalidToken, "format"));
        }

        let app_id =
            u64::from_str_radix(parts[0], 10).map_err(|_| api_error(ApiErrorCode::InvalidToken))?;
        let user_id =
            u64::from_str_radix(parts[1], 10).map_err(|_| api_error(ApiErrorCode::InvalidToken))?;

        return Ok(Self { app_id, user_id });
    }
}

pub fn build_user_token(claims: UserClaims) -> Result<String, ApiError> {
    let mut out = String::new();
    out.push_str(&claims.app_id.to_string());
    out.push_str("@");
    out.push_str(&claims.user_id.to_string());
    return jwt_build(SDK_TC, out);
}
