use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
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

    let app = app::get_by_id(&mut conn, payload.app_id).await?;

    let mut topic = topic::TopicModel {
        app_id: app.id,
        user_id: claims.user_id,
        category: payload.category,
        title: payload.title,
        content: payload.content,
        ..Default::default()
    };

    let topic_id = topic::create(&mut conn, &mut topic).await?;
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

    Ok(api_success(TopicDetailResponse {
        topic: topic.to_simple(),
    }))
}
