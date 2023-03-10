use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use super::base::*;
use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

async fn root() -> &'static str {
    "DoggTalk SDK Topic API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(topic_create))
        .route("/detail", get(topic_detail))
        .route("/list", get(topic_list))
}

#[derive(Deserialize)]
struct TopicCreatePayload {
    app_id: u64,
    category: u64,
    title: String,
    content: String,
}

#[derive(Serialize)]
struct TopicCreateResponse {
    topic: topic::TopicSimple,
}

async fn topic_create(
    claims: UserClaims,
    Json(payload): Json<TopicCreatePayload>,
) -> Result<ApiSuccess<TopicCreateResponse>, ApiError> {
    let mut conn = database_connect().await?;

    if payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let user = user::get_by_id(&mut conn, claims.user_id).await?;
    if payload.app_id != user.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !user.is_actived() {
        return Err(api_error(ApiErrorCode::AccountNotActived));
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
    user::update_topic_count(&mut conn, claims.user_id).await?;

    let topic = topic::get_by_id(&mut conn, topic_id).await?;

    Ok(api_success(TopicCreateResponse {
        topic: topic.to_simple(),
    }))
}

#[derive(Deserialize)]
struct TopicDetailPayload {
    app_id: u64,
    topic_id: u64,
}

#[derive(Serialize)]
struct TopicDetailResponse {
    topic: topic::TopicSimple,
}

async fn topic_detail(
    Query(payload): Query<TopicDetailPayload>,
) -> Result<ApiSuccess<TopicDetailResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if topic.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !topic.is_actived() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    Ok(api_success(TopicDetailResponse {
        topic: topic.to_simple(),
    }))
}

#[derive(Deserialize)]
struct TopicListPayload {
    app_id: u64,
    category: u64,
    order_by: topic::VisiblesOrderBy,
    cursor: u32,
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
    Query(payload): Query<TopicListPayload>,
) -> Result<ApiSuccess<TopicListResponse>, ApiError> {
    let mut conn = database_connect().await?;

    app::get_by_id(&mut conn, payload.app_id).await?;

    let (total, topics) = topic::fetch_visibles(
        &mut conn,
        payload.app_id,
        payload.category,
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
            user: user_map
                .get(&s.user_id)
                .unwrap_or(&user::DEFAULT_SIMPLE)
                .clone(),
            topic: s.to_simple(),
        })
        .collect();

    Ok(api_success(TopicListResponse { total, topics }))
}
