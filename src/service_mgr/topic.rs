use axum::{
    routing::{get, post},
    Router,
};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk MGR Topic API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(topic_create))
        .route("/detail", get(topic_detail))
        .route("/list", get(topic_list))
        .route("/update/status", post(topic_update_status))
}

#[derive(Deserialize)]
struct TopicCreatePayload {
    app_id: u64,
    user_id: u64,
    category: u64,
    title: String,
    content: String,
}

#[derive(Serialize)]
struct TopicCreateResponse {
    topic_id: u64,
}

async fn topic_create(
    _claims: MgrClaims,
    Json(payload): Json<TopicCreatePayload>,
) -> Result<ApiSuccess<TopicCreateResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, payload.user_id).await?;
    if payload.app_id != user.app_id || user.source != user::SOURCE_FAKE {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let mut topic = topic::TopicModel {
        app_id: user.app_id,
        user_id: user.id,
        category: payload.category,
        title: payload.title,
        content: payload.content,
        ..Default::default()
    };

    let topic_id = topic::create(&mut conn, &mut topic).await?;
    user::update_topic_count(&mut conn, user.id, UpdateCountOp::INCR).await?;

    let topic = topic::get_by_id(&mut conn, topic_id).await?;

    Ok(api_success(TopicCreateResponse { topic_id: topic.id }))
}

#[derive(Deserialize)]
struct TopicDetailPayload {
    app_id: u64,
    topic_id: u64,
}

#[derive(Serialize)]
struct TopicDetailResponse {
    user: user::UserSimple,
    topic: topic::TopicSimple,
}

async fn topic_detail(
    _claims: MgrClaims,
    Query(payload): Query<TopicDetailPayload>,
) -> Result<ApiSuccess<TopicDetailResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if topic.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if topic.is_deleted() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    let user = user::get_by_id(&mut conn, topic.user_id).await?;

    Ok(api_success(TopicDetailResponse {
        user: user.to_simple(),
        topic: topic.to_simple(),
    }))
}

#[derive(Validate, Deserialize)]
struct TopicListPayload {
    app_id: u64,
    category: u64,
    style: topic::VisibleStyle,
    order_by: topic::VisibleOrderBy,
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct TopicListItem {
    topic: topic::TopicSimple,
    user: user::ArcUserSimple,
}

#[derive(Serialize)]
struct TopicListResponse {
    total: u32,
    topics: Vec<TopicListItem>,
}

async fn topic_list(
    _claims: MgrClaims,
    Query(payload): Query<TopicListPayload>,
) -> Result<ApiSuccess<TopicListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    app::get_by_id(&mut conn, payload.app_id).await?;

    let (total, topics) = topic::fetch_more(
        &mut conn,
        payload.app_id,
        payload.category,
        payload.style,
        payload.order_by,
        payload.cursor,
        payload.count,
    )
    .await?;

    let user_map =
        user::get_simple_map_by_ids(&mut conn, topics.iter().map(|s| s.user_id).collect()).await?;

    let topics = topics
        .iter()
        .map(|s| TopicListItem {
            user: user_map.get(s.user_id),
            topic: s.to_simple(),
        })
        .collect();

    Ok(api_success(TopicListResponse { total, topics }))
}

#[derive(Deserialize)]
struct TopicUpdateStatusPayload {
    app_id: u64,
    topic_id: u64,
    action: topic::StatusAction,
}

#[derive(Serialize)]
struct TopicUpdateStatusResponse {
    topic_id: u64,
}

async fn topic_update_status(
    _claims: MgrClaims,
    Json(payload): Json<TopicUpdateStatusPayload>,
) -> Result<ApiSuccess<TopicUpdateStatusResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if topic.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if topic.is_deleted() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    topic::update_status(&mut conn, topic.id, payload.action.clone()).await?;
    if payload.action == topic::StatusAction::DELETE {
        user::update_topic_count(&mut conn, topic.user_id, UpdateCountOp::DECR).await?;
    }

    Ok(api_success(TopicUpdateStatusResponse {
        topic_id: topic.id,
    }))
}
