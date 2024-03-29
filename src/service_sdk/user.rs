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
    "DoggTalk SDK User API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/login/sync", post(user_sync_login))
        .route("/detail", get(user_detail))
        .route("/update/profile", post(user_update_profile))
}

#[derive(Validate, Deserialize)]
struct UserSyncLoginPayload {
    app_id: u64,
    account: String,
    display_name: String,
    #[validate(custom = "validate_url")]
    avatar_url: Option<String>,
    #[validate(custom = "validate_gender")]
    gender: i8,
    safe_sign: String,
}

#[derive(Serialize)]
struct UserSyncLoginResponse {
    token: String,
    user: user::UserSimple,
}

async fn user_sync_login(
    Json(payload): Json<UserSyncLoginPayload>,
) -> Result<ApiSuccess<UserSyncLoginResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let app = app::get_by_id(&mut conn, payload.app_id).await?;
    let mut safe_text = String::new();
    safe_text.push_str(&payload.app_id.to_string());
    safe_text.push_str(&app.app_secret);
    safe_text.push_str(&payload.account);
    safe_text.push_str(&app.app_secret);
    if !verify_hash(&safe_text, &payload.safe_sign) {
        return Err(api_error(ApiErrorCode::InvalidSign));
    }

    let exists_user = user::get_by_account(
        &mut conn,
        payload.app_id,
        user::SOURCE_SYNC,
        &payload.account,
    )
    .await?;

    let mut user: user::UserModel;
    if exists_user.is_none() {
        user = user::UserModel {
            app_id: app.id,
            source: user::SOURCE_SYNC,
            account: payload.account,
            display_name: payload.display_name,
            avatar_url: payload.avatar_url,
            gender: payload.gender,
            ..Default::default()
        };

        let user_id = user::create(&mut conn, &mut user).await?;
        user = user::get_by_id(&mut conn, user_id).await?;
    } else {
        user = exists_user.unwrap();
        if user.try_update_profile(payload.display_name, payload.avatar_url, payload.gender) {
            user::update_profile(&mut conn, &mut user).await?
        }
    }

    Ok(api_success(UserSyncLoginResponse {
        token: build_user_token(UserClaims {
            app_id: user.app_id,
            user_id: user.id,
        })?,
        user: user.to_simple(),
    }))
}

#[derive(Serialize)]
struct UserDetailResponse {
    user: user::UserModel,
}

async fn user_detail(claims: UserClaims) -> Result<ApiSuccess<UserDetailResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, claims.user_id).await?;

    Ok(api_success(UserDetailResponse { user }))
}

#[derive(Validate, Deserialize)]
struct UserUpdateProfilePayload {
    display_name: String,
    #[validate(custom = "validate_url")]
    avatar_url: Option<String>,
    #[validate(custom = "validate_gender")]
    gender: i8,
}

async fn user_update_profile(
    claims: UserClaims,
    Json(payload): Json<UserUpdateProfilePayload>,
) -> Result<ApiSuccess<UserDetailResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let mut user = user::get_by_id(&mut conn, claims.user_id).await?;
    if user.try_update_profile(payload.display_name, payload.avatar_url, payload.gender) {
        user::update_profile(&mut conn, &mut user).await?;
    }

    Ok(api_success(UserDetailResponse { user }))
}
