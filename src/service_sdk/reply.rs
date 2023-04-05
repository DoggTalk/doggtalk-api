use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk SDK Reply API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(reply_create))
        .route("/list", get(reply_list))
}

#[derive(Deserialize)]
struct ReplyCreatePayload {
    app_id: u64,
    topic_id: u64,
    content: String,
}

#[derive(Serialize)]
struct ReplyCreateResponse {
    reply_id: u64,
}

async fn reply_create(
    claims: UserClaims,
    Json(payload): Json<ReplyCreatePayload>,
) -> Result<ApiSuccess<ReplyCreateResponse>, ApiError> {
    let mut conn = database_connect().await?;

    if payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if payload.app_id != topic.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !topic.is_actived() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    let user = user::get_by_id(&mut conn, claims.user_id).await?;
    if payload.app_id != user.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !user.is_actived() {
        return Err(api_error(ApiErrorCode::AccountNotActived));
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
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct ReplyListItem {
    reply: reply::ReplySimple,
    user: user::ArcUserSimple,
}

#[derive(Serialize)]
struct ReplyListResponse {
    total: u32,
    replies: Vec<ReplyListItem>,
}

async fn reply_list(
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

    let (total, replies) = reply::fetch_more(
        &mut conn,
        topic.id,
        reply::VisibleStyle::NORMAL,
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
