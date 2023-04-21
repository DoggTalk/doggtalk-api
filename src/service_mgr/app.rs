use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk MGR App API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(app_create))
        .route("/detail", get(app_detail))
        .route("/list", get(app_list))
}

#[derive(Validate, Deserialize)]
struct AppCreatePayload {
    name: String,
    #[validate(custom = "validate_url")]
    icon_url: Option<String>,
}

#[derive(Serialize)]
struct AppCreateResponse {
    app_id: u64,
}

async fn app_create(
    _claims: MgrClaims,
    Json(payload): Json<AppCreatePayload>,
) -> Result<ApiSuccess<AppCreateResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let app = app::AppModel {
        app_key: app::build_key(),
        app_secret: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        icon_url: payload.icon_url,
        ..Default::default()
    };

    let res = app::create(&mut conn, app).await?;

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
    let mut conn = database_connect().await?;

    let app = app::get_by_id(&mut conn, payload.app_id).await?;

    Ok(api_success(AppDetailResponse { app }))
}

#[derive(Validate, Deserialize)]
struct AppListPayload {
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct AppListResponse {
    total: u32,
    apps: Vec<app::AppSimple>,
}

async fn app_list(
    _claims: MgrClaims,
    Query(payload): Query<AppListPayload>,
) -> Result<ApiSuccess<AppListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let (total, apps) = app::fetch_pagging(&mut conn, payload.cursor, payload.count).await?;

    let apps = apps.iter().map(|s| s.to_simple()).collect();

    Ok(api_success(AppListResponse { total, apps }))
}
