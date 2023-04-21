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
    "DoggTalk SDK Topic API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(topic_create))
        .route("/detail", get(topic_detail))
        .route("/like", post(topic_like))
        .route("/unlike", post(topic_unlike))
        .route("/list", get(topic_list))
}

fn like_key(id: u64) -> String {
    format!("topiclike:{}", id)
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
    topic_id: u64,
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
    user::update_topic_count(&mut conn, claims.user_id, UpdateCountOp::INCR).await?;

    let topic = topic::get_by_id(&mut conn, topic_id).await?;

    Ok(api_success(TopicCreateResponse { topic_id: topic.id }))
}

#[derive(Deserialize)]
struct TopicLikePayload {
    app_id: u64,
    topic_id: u64,
}

#[derive(Serialize)]
struct TopicLikeResponse {
    affect: u64,
    like_count: u64,
}

async fn topic_like(
    claims: UserClaims,
    Json(payload): Json<TopicLikePayload>,
) -> Result<ApiSuccess<TopicLikeResponse>, ApiError> {
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

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if topic.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !topic.is_actived() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    let mut connr = redis_connect().await?;

    let affect: u64 = redis::cmd("ZADD")
        .arg(like_key(topic.id))
        .arg("NX")
        .arg(timestamp())
        .arg(user.id)
        .query_async(&mut *connr)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if affect > 0 {
        topic::update_like_count(&mut conn, topic.id, UpdateCountOp::INCR).await?;
    }

    Ok(api_success(TopicLikeResponse {
        affect,
        like_count: topic.like_count + affect,
    }))
}

async fn topic_unlike(
    claims: UserClaims,
    Json(payload): Json<TopicLikePayload>,
) -> Result<ApiSuccess<TopicLikeResponse>, ApiError> {
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

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if topic.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !topic.is_actived() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    let mut connr = redis_connect().await?;

    let affect: u64 = redis::cmd("ZREM")
        .arg(like_key(topic.id))
        .arg(user.id)
        .query_async(&mut *connr)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if affect > 0 {
        topic::update_like_count(&mut conn, topic.id, UpdateCountOp::DECR).await?;
    }

    Ok(api_success(TopicLikeResponse {
        affect,
        like_count: topic.like_count - affect,
    }))
}

#[derive(Serialize)]
struct MySelfData {
    is_liked: bool,
}

impl Default for MySelfData {
    fn default() -> Self {
        Self { is_liked: false }
    }
}

async fn fetch_myself<C>(
    conn: &mut C,
    user_id: u64,
    topic_ids: Vec<u64>,
) -> Result<ArcDataMap<u64, MySelfData>, ApiError>
where
    C: RedisConnectionLike,
{
    let mut pipe = redis::pipe();
    for id in topic_ids.iter() {
        pipe.cmd("ZSCORE").arg(like_key(*id)).arg(user_id);
    }
    let scores: Vec<Option<u64>> = pipe
        .query_async(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let mut like_ids = HashSet::new();
    for (i, o) in scores.iter().enumerate() {
        if o.is_some() {
            like_ids.insert(topic_ids.get(i).unwrap());
        }
    }

    let mut out = ArcDataMap::new();
    for id in topic_ids.iter() {
        out.insert(
            *id,
            MySelfData {
                is_liked: like_ids.contains(id),
            },
        );
    }

    Ok(out)
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
    myself: Option<Arc<MySelfData>>,
}

async fn topic_detail(
    claims: Option<UserClaims>,
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

    let user = user::get_by_id(&mut conn, topic.user_id).await?;

    let mut myself = None;
    let user_id = claims.unwrap_or_default().user_id;
    if user_id > 0 {
        let mut connr = redis_connect().await?;

        let myself_map = fetch_myself(&mut *connr, user_id, vec![topic.id]).await?;

        myself = Some(myself_map.get(topic.id))
    }

    Ok(api_success(TopicDetailResponse {
        user: user.to_simple(),
        topic: topic.to_simple(),
        myself,
    }))
}

#[derive(Validate, Deserialize)]
struct TopicListPayload {
    app_id: u64,
    category: u64,
    order_by: topic::VisibleOrderBy,
    cursor: u32,
    #[validate(custom = "validate_page_count")]
    count: u32,
}

#[derive(Serialize)]
struct TopicListItem {
    topic: topic::TopicSimple,
    user: Arc<user::UserSimple>,
    myself: Option<Arc<MySelfData>>,
}

#[derive(Serialize)]
struct TopicListResponse {
    total: u32,
    topics: Vec<TopicListItem>,
}

async fn topic_list(
    claims: Option<UserClaims>,
    Query(payload): Query<TopicListPayload>,
) -> Result<ApiSuccess<TopicListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let mut conn = database_connect().await?;

    app::get_by_id(&mut conn, payload.app_id).await?;

    let (total, topics) = topic::fetch_pagging(
        &mut conn,
        payload.app_id,
        payload.category,
        topic::VisibleStyle::NORMAL,
        payload.order_by,
        payload.cursor,
        payload.count,
    )
    .await?;

    let user_map =
        user::get_simple_map_by_ids(&mut conn, topics.iter().map(|s| s.user_id).collect()).await?;

    let mut myself_map = ArcDataMap::new();

    let user_id = claims.unwrap_or_default().user_id;
    if user_id > 0 && !topics.is_empty() {
        let mut connr = redis_connect().await?;

        myself_map =
            fetch_myself(&mut *connr, user_id, topics.iter().map(|s| s.id).collect()).await?;
    }

    let topics = topics
        .iter()
        .map(|s| {
            let mut myself = None;
            if user_id > 0 {
                myself = Some(myself_map.get(s.id));
            }

            TopicListItem {
                user: user_map.get(s.user_id),
                topic: s.to_simple(),
                myself,
            }
        })
        .collect();

    Ok(api_success(TopicListResponse { total, topics }))
}
