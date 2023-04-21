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
    "DoggTalk SDK Reply API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/create", post(reply_create))
        .route("/like", post(reply_like))
        .route("/unlike", post(reply_unlike))
        .route("/list", get(reply_list))
}

fn like_key(id: u64) -> String {
    format!("replylike:{}", id)
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
    if payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let mut conn = database_connect().await?;

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

#[derive(Deserialize)]
struct ReplyLikePayload {
    app_id: u64,
    reply_id: u64,
}

#[derive(Serialize)]
struct ReplyLikeResponse {
    affect: u64,
    like_count: u64,
}

async fn reply_like(
    claims: UserClaims,
    Json(payload): Json<ReplyLikePayload>,
) -> Result<ApiSuccess<ReplyLikeResponse>, ApiError> {
    if payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, claims.user_id).await?;
    if payload.app_id != user.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !user.is_actived() {
        return Err(api_error(ApiErrorCode::AccountNotActived));
    }

    let reply = reply::get_by_id(&mut conn, payload.reply_id).await?;
    if reply.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !reply.is_actived() {
        return Err(api_error(ApiErrorCode::ReplyNotFound));
    }

    let mut connr = redis_connect().await?;

    let affect: u64 = redis::cmd("ZADD")
        .arg(like_key(reply.id))
        .arg("NX")
        .arg(timestamp())
        .arg(user.id)
        .query_async(&mut *connr)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if affect > 0 {
        reply::update_like_count(&mut conn, reply.id, UpdateCountOp::INCR).await?;
    }

    Ok(api_success(ReplyLikeResponse {
        affect,
        like_count: reply.like_count + affect,
    }))
}

async fn reply_unlike(
    claims: UserClaims,
    Json(payload): Json<ReplyLikePayload>,
) -> Result<ApiSuccess<ReplyLikeResponse>, ApiError> {
    if payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let mut conn = database_connect().await?;

    let user = user::get_by_id(&mut conn, claims.user_id).await?;
    if payload.app_id != user.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !user.is_actived() {
        return Err(api_error(ApiErrorCode::AccountNotActived));
    }

    let reply = reply::get_by_id(&mut conn, payload.reply_id).await?;
    if reply.app_id != payload.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }
    if !reply.is_actived() {
        return Err(api_error(ApiErrorCode::ReplyNotFound));
    }

    let mut connr = redis_connect().await?;

    let affect: u64 = redis::cmd("ZREM")
        .arg(like_key(reply.id))
        .arg(user.id)
        .query_async(&mut *connr)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if affect > 0 {
        reply::update_like_count(&mut conn, reply.id, UpdateCountOp::DECR).await?;
    }

    Ok(api_success(ReplyLikeResponse {
        affect,
        like_count: reply.like_count + affect,
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
    reply_ids: Vec<u64>,
) -> Result<ArcDataMap<u64, MySelfData>, ApiError>
where
    C: RedisConnectionLike,
{
    let mut pipe = redis::pipe();
    for id in reply_ids.iter() {
        pipe.cmd("ZSCORE").arg(like_key(*id)).arg(user_id);
    }
    let scores: Vec<Option<u64>> = pipe
        .query_async(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let mut like_ids = HashSet::new();
    for (i, o) in scores.iter().enumerate() {
        if o.is_some() {
            like_ids.insert(reply_ids.get(i).unwrap());
        }
    }

    let mut out = ArcDataMap::new();
    for id in reply_ids.iter() {
        out.insert(
            *id,
            MySelfData {
                is_liked: like_ids.contains(id),
            },
        );
    }

    Ok(out)
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
    user: Arc<user::UserSimple>,
    myself: Option<Arc<MySelfData>>,
}

#[derive(Serialize)]
struct ReplyListResponse {
    total: u32,
    replies: Vec<ReplyListItem>,
}

async fn reply_list(
    claims: Option<UserClaims>,
    Query(payload): Query<ReplyListPayload>,
) -> Result<ApiSuccess<ReplyListResponse>, ApiError> {
    match payload.validate() {
        Err(e) => return Err(api_errore(ApiErrorCode::InvalidParams, &e)),
        _ => {}
    };

    let claims = claims.unwrap_or_default();
    if claims.app_id != 0 && payload.app_id != claims.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let mut conn = database_connect().await?;

    let topic = topic::get_by_id(&mut conn, payload.topic_id).await?;
    if payload.app_id != topic.app_id {
        return Err(api_error(ApiErrorCode::NoPermission));
    }

    let (total, replies) = reply::fetch_pagging(
        &mut conn,
        topic.id,
        reply::VisibleStyle::NORMAL,
        payload.cursor,
        payload.count,
    )
    .await?;

    let user_map =
        user::get_simple_map_by_ids(&mut conn, replies.iter().map(|s| s.user_id).collect()).await?;

    let myself_map = if claims.user_id == 0 || replies.is_empty() {
        ArcDataMap::new()
    } else {
        let mut connr = redis_connect().await?;

        fetch_myself(
            &mut *connr,
            claims.user_id,
            replies.iter().map(|s| s.id).collect(),
        )
        .await?
    };

    let replies = replies
        .iter()
        .map(|s| {
            let myself = if claims.user_id == 0 {
                None
            } else {
                myself_map.opt(s.id)
            };

            ReplyListItem {
                user: user_map.get(s.user_id),
                reply: s.to_simple(),
                myself,
            }
        })
        .collect();

    Ok(api_success(ReplyListResponse { total, replies }))
}
