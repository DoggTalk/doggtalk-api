use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;
use serde::{Deserialize, Serialize};

async fn root() -> &'static str {
    "DoggTalk MGR App API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(app_create))
        .route("/detail", get(app_detail))
        .route("/list", get(app_list))
        .route("/list/all", get(app_list_all))
}

#[derive(Deserialize)]
struct AppCreatePayload {
    name: String,
    icon_url: String,
}

#[derive(Serialize)]
struct AppCreateResponse {
    app_id: u64,
}

async fn app_create(
    Json(payload): Json<AppCreatePayload>,
) -> Result<ApiSuccess<AppCreateResponse>, ApiError> {
    let conn = database_connect().await?;

    let app = app::AppModel {
        id: 0,
        app_key: app::build_key(),
        app_secret: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        icon_url: payload.icon_url,
        created_at: SqlDateTime::MIN,
    };

    let res = app::create(conn, app).await?;

    Ok(api_success(AppCreateResponse { app_id: res }))
}

#[derive(Deserialize)]
struct AppDetailPayload {
    app_id: u64,
}

#[derive(Serialize)]
struct AppDetailResponse {
    app: app::AppModel,
}

async fn app_detail(
    _claims: MgrClaims,
    Query(payload): Query<AppDetailPayload>,
) -> Result<ApiSuccess<AppDetailResponse>, ApiError> {
    let conn = database_connect().await?;

    let app = app::get_by_id(conn, payload.app_id).await?;

    Ok(api_success(AppDetailResponse { app }))
}

#[derive(Deserialize)]
struct AppListPayload {
    cursor: u32,
    count: u32,
}

#[derive(Serialize)]
struct AppListResponse {
    total: u32,
    apps: Vec<app::AppModel>,
}

async fn app_list(
    _claims: MgrClaims,
    Query(payload): Query<AppListPayload>,
) -> Result<ApiSuccess<AppListResponse>, ApiError> {
    let conn = database_connect().await?;

    let (total, apps) = app::fetch_more(conn, payload.cursor, payload.count).await?;

    Ok(api_success(AppListResponse { total, apps }))
}

#[derive(Serialize)]
struct AppListAllResponse {
    apps: Vec<app::AppSimple>,
}

async fn app_list_all(_claims: MgrClaims) -> Result<ApiSuccess<AppListAllResponse>, ApiError> {
    let conn = database_connect().await?;

    let apps = app::fetch_simple_all(conn).await?;

    Ok(api_success(AppListAllResponse { apps }))
}
