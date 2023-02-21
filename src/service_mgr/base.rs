use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};

use crate::shared::jwt::*;
use crate::shared::web::*;

const MGR_TC: &'static str = "mgr";

pub struct MgrClaims {
    pub mgr_id: u64,
}

#[async_trait]
impl<S> FromRequestParts<S> for MgrClaims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| api_error(ApiErrorCode::InvalidToken))?;

        let res = jwt_parse(MGR_TC, bearer.token())?;
        let mgr_id =
            u64::from_str_radix(&res, 10).map_err(|_| api_error(ApiErrorCode::InvalidToken))?;

        return Ok(MgrClaims { mgr_id });
    }
}

pub fn build_mgr_token(claims: MgrClaims) -> Result<String, ApiError> {
    return jwt_build(MGR_TC, claims.mgr_id.to_string());
}
