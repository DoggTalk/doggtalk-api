use axum::{
    routing::{get, post},
    Json, Router,
};

use super::base::*;
use crate::shared::web::*;
use serde::{Deserialize, Serialize};

async fn root() -> &'static str {
    "DoggTalk MGR Manager API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/login", post(manager_login))
        .route("/detail", get(manager_detail))
}

#[derive(Deserialize)]
struct ManagerLoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ManagerLoginResponse {
    token: String,
}

async fn manager_login(
    Json(payload): Json<ManagerLoginPayload>,
) -> Result<ApiSuccess<ManagerLoginResponse>, ApiError> {
    if !payload.username.eq("admin") || !payload.password.eq("admin") {
        return Err(api_error(ApiErrorCode::UserOrPasswordFailed));
    }

    let token = build_mgr_token(MgrClaims { mgr_id: 999 })?;

    Ok(api_success(ManagerLoginResponse { token }))
}

#[derive(Serialize)]
struct ManagerDetailResponse {
    id: u64,
    username: String,
}

async fn manager_detail(claims: MgrClaims) -> Result<ApiSuccess<ManagerDetailResponse>, ApiError> {
    Ok(api_success(ManagerDetailResponse {
        id: claims.mgr_id,
        username: String::from("admin"),
    }))
}
