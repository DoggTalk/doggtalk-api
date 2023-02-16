use axum::{
    routing::{get, post},
    Json, Router,
};

use crate::common::web::*;
use serde::{Deserialize, Serialize};

async fn root() -> &'static str {
    "DoggTalk MGR Manager API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/login", post(manager_login))
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
        return Err(app_error(ApiErrorCode::UserOrPasswordFailed));
    }

    let res = ManagerLoginResponse {
        token: String::from("ok"),
    };

    Ok(app_success(res))
}
