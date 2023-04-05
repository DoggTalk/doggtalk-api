use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk MGR User API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(user_create))
        .route("/detail", get(user_detail))
        .route("/list", get(user_list))
        .route("/update/profile", post(user_update_profile))
}

#[derive(Validate, Deserialize)]
struct UserCreatePayload {
    app_id: u64,
    display_name: String,
    #[validate(custom = "validate_url")]
    avatar_url: Option<String>,
    #[validate(custom = "validate_gender")]
    gender: i8,
}

#[derive(Serialize)]
struct UserCreateResponse {
    user_id: u64,
}

async fn user_create(
    _claims: MgrClaims,
    Json(payload): Json<UserCreatePayload>,
) -> Result<ApiSuccess<UserCreateResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let app = app::get_by_id(&mut conn, payload.app_id).await?;

    let mut user = user::UserModel {
        app_id: app.id,
        source: user::SOURCE_FAKE,
        account: uuid::Uuid::new_v4().to_string(),
        display_name: payload.display_name,
        avatar_url: payload.avatar_url,
        gender: payload.gender,
        ..Default::default()
    };

    let user_id = user::create(&mut conn, &mut user).await?;

    Ok(api_success(UserCreateResponse { user_id }))
}

#[derive(Deserialize)]
struct UserDetailPayload {
    app_id: u64,
    user_id: u64,
}

#[derive(Serialize)]
struct UserDetailResponse {
    user: user::UserModel,
}

async fn user_detail(
    _claims: MgrClaims,
    Query(payload): Query<UserDetailPayload>,
) -> Result<ApiSuccess<UserDetailResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, payload.user_id).await?;
    if payload.app_id != user.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    Ok(api_success(UserDetailResponse { user }))
}

#[derive(Validate, Deserialize)]
struct UserUpdateProfilePayload {
    app_id: u64,
    user_id: u64,
    display_name: String,
    #[validate(custom = "validate_url")]
    avatar_url: Option<String>,
    #[validate(custom = "validate_gender")]
    gender: i8,
}

async fn user_update_profile(
    _claims: MgrClaims,
    Json(payload): Json<UserUpdateProfilePayload>,
) -> Result<ApiSuccess<UserDetailResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let mut user = user::get_by_id(&mut conn, payload.user_id).await?;
    if payload.app_id != user.app_id || user.source != user::SOURCE_FAKE {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    if user.try_update_profile(payload.display_name, payload.avatar_url, payload.gender) {
        user::update_profile(&mut conn, &mut user).await?;
    }

    Ok(api_success(UserDetailResponse { user }))
}

#[derive(Validate, Deserialize)]
struct UserListPayload {
    app_id: u64,
    #[validate(range(min = -1, max = 1))]
    source: i8,
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct UserListResponse {
    total: u32,
    users: Vec<user::UserSimple>,
}

async fn user_list(
    _claims: MgrClaims,
    Query(payload): Query<UserListPayload>,
) -> Result<ApiSuccess<UserListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let (total, users) = user::fetch_more(
        &mut conn,
        payload.app_id,
        payload.source,
        payload.cursor,
        payload.count,
    )
    .await?;

    let users = users.iter().map(|s| s.to_simple()).collect();

    Ok(api_success(UserListResponse { total, users }))
}
