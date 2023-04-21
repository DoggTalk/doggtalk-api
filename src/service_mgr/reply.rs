use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk MGR Reply API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(reply_create))
        .route("/list", get(reply_list))
        .route("/update/status", post(reply_update_status))
}

#[derive(Deserialize)]
struct ReplyCreatePayload {
    app_id: u64,
    user_id: u64,
    topic_id: u64,
    content: String,
}

#[derive(Serialize)]
struct ReplyCreateResponse {
    reply_id: u64,
}

async fn reply_create(
    _claims: MgrClaims,
    Json(payload): Json<ReplyCreatePayload>,
) -> Result<ApiSuccess<ReplyCreateResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, payload.user_id).await?;
    if payload.app_id != user.app_id || user.source != user::SOURCE_FAKE {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if payload.app_id != topic.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !topic.is_actived() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    let mut reply = reply::ReplyModel {
        app_id: topic.app_id,
        topic_id: topic.id,
        user_id: user.id,
        content: payload.content,
        ..Default::default()
    };

    let reply_id = reply::create(&mut conn, &mut reply).await?;
    topic::update_reply_count(&mut conn, topic.id, UpdateCountOp::INCR).await?;

    let reply = reply::get_by_id(&mut conn, reply_id).await?;

    Ok(api_success(ReplyCreateResponse { reply_id: reply.id }))
}

#[derive(Validate, Deserialize)]
struct ReplyListPayload {
    app_id: u64,
    topic_id: u64,
    style: reply::VisibleStyle,
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct ReplyListItem {
    reply: reply::ReplySimple,
    user: Arc<user::UserSimple>,
}

#[derive(Serialize)]
struct ReplyListResponse {
    total: u32,
    replies: Vec<ReplyListItem>,
}

async fn reply_list(
    _claims: MgrClaims,
    Query(payload): Query<ReplyListPayload>,
) -> Result<ApiSuccess<ReplyListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if payload.app_id != topic.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let (total, replies) = reply::fetch_pagging(
        &mut conn,
        topic.id,
        payload.style,
        payload.cursor,
        payload.count,
    )
    .await?;

    let user_map =
        user::get_simple_map_by_ids(&mut conn, replies.iter().map(|s| s.user_id).collect()).await?;

    let replies = replies
        .iter()
        .map(|s| ReplyListItem {
            user: user_map.get(s.user_id),
            reply: s.to_simple(),
        })
        .collect();

    Ok(api_success(ReplyListResponse { total, replies }))
}

#[derive(Deserialize)]
struct ReplyUpdateStatusPayload {
    app_id: u64,
    reply_id: u64,
    action: reply::StatusAction,
}

#[derive(Serialize)]
struct ReplyUpdateStatusResponse {
    reply_id: u64,
}

async fn reply_update_status(
    _claims: MgrClaims,
    Json(payload): Json<ReplyUpdateStatusPayload>,
) -> Result<ApiSuccess<ReplyUpdateStatusResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let reply = reply::get_by_id(&mut conn, payload.reply_id).await?;
    if reply.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if reply.is_deleted() {
        return Err(api_error(ApiErrorCode::ReplyNotFound));
    }

    reply::update_status(&mut conn, reply.id, payload.action.clone()).await?;
    if payload.action == reply::StatusAction::DELETE {
        topic::update_reply_count(&mut conn, reply.topic_id, UpdateCountOp::DECR).await?;
    }

    Ok(api_success(ReplyUpdateStatusResponse {
        reply_id: reply.id,
    }))
}
