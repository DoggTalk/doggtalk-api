use serde::Serialize;

use crate::shared::data::*;
use crate::shared::web::*;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplyModel {
    pub id: u64,
    pub app_id: u64,
    pub topic_id: u64,
    pub user_id: u64,
    pub content: String,
    pub topped: i64,
    pub created_at: SqlDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplySimple {
    pub id: u64,
    pub user_id: u64,
    pub content: String,
    pub topped: i64,
    pub created_at: SqlDateTime,
}

impl ReplyModel {
    pub fn to_simple(self: &Self) -> ReplySimple {
        ReplySimple {
            id: self.id,
            user_id: self.user_id,
            content: self.content.clone(),
            topped: self.topped,
            created_at: self.created_at,
        }
    }
}

impl Default for ReplyModel {
    fn default() -> ReplyModel {
        ReplyModel {
            id: 0,
            app_id: 0,
            topic_id: 0,
            user_id: 0,
            content: String::new(),
            topped: 0,
            created_at: SqlDateTime::MIN,
        }
    }
}

pub async fn get_by_id(conn: &mut SqlConnection, id: u64) -> Result<ReplyModel, ApiError> {
    let res = sqlx::query_as::<_, ReplyModel>("select * from dg_replies where id=?")
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::ReplyNotFound));
    }

    Ok(res.unwrap())
}

pub async fn create(conn: &mut SqlConnection, reply: &mut ReplyModel) -> Result<u64, ApiError> {
    let res = sqlx::query(
        "insert into dg_replies(app_id,topic_id,user_id,content,topped) values(?,?,?,?,0)",
    )
    .bind(reply.app_id)
    .bind(reply.topic_id)
    .bind(reply.user_id)
    .bind(&reply.content)
    .execute(conn)
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res.last_insert_id())
}

pub async fn fetch_visibles(
    conn: &mut SqlConnection,
    topic_id: u64,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<ReplyModel>), ApiError> {
    let replies = sqlx::query_as::<_, ReplyModel>(
        "select * from dg_replies where topic_id=? and topped>=0 order by topped desc,created_at desc limit ?,?",
    )
    .bind(topic_id)
    .bind(cursor)
    .bind(count)
    .fetch_all(conn.as_mut())
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let total: (i64,) = sqlx::query_as("select count(*) from dg_replies where topic_id=?")
        .bind(topic_id)
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok((total.0 as u32, replies))
}
