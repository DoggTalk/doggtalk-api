use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::hash::*;
use crate::shared::model::*;
use crate::shared::web::*;

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
    let mut conn = database_connect().await?;

    let manager = manager::get_by_username(&mut conn, &payload.username).await?;
    if manager.is_none() {
        return Err(api_error(ApiErrorCode::AccountOrPasswordFailed));
    }

    let manager = manager.unwrap();
    if !verify_hash(&payload.password, &manager.password) {
        return Err(api_error(ApiErrorCode::AccountOrPasswordFailed));
    }

    Ok(api_success(ManagerLoginResponse {
        token: build_mgr_token(MgrClaims { mgr_id: manager.id })?,
    }))
}

#[derive(Serialize)]
struct ManagerDetailResponse {
    manager: manager::ManagerModel,
}

async fn manager_detail(claims: MgrClaims) -> Result<ApiSuccess<ManagerDetailResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let manager = manager::get_by_id(&mut conn, claims.mgr_id).await?;

    Ok(api_success(ManagerDetailResponse { manager }))
}
