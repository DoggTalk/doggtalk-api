use serde::{Deserialize, Serialize};

use crate::shared::data::*;
use crate::shared::web::*;

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisiblesOrderBy {
    CREATE = 0,
    REFRESH = 1,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TopicModel {
    pub id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub category: u64,
    pub title: String,
    pub content: String,
    pub topped: u64,
    pub reply_count: u64,
    pub created_at: SqlDateTime,
    pub refreshed_at: SqlDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TopicSimple {
    pub id: u64,
    pub user_id: u64,
    pub category: u64,
    pub title: String,
    pub content: String,
    pub topped: u64,
    pub reply_count: u64,
    pub created_at: SqlDateTime,
    pub refreshed_at: SqlDateTime,
}

impl TopicModel {
    pub fn to_simple(self: &Self) -> TopicSimple {
        TopicSimple {
            id: self.id,
            user_id: self.user_id,
            category: self.category,
            title: self.title.clone(),
            content: self.content.clone(),
            topped: self.topped,
            reply_count: self.reply_count,
            created_at: self.created_at,
            refreshed_at: self.refreshed_at,
        }
    }
}

impl Default for TopicModel {
    fn default() -> TopicModel {
        TopicModel {
            id: 0,
            app_id: 0,
            user_id: 0,
            category: 0,
            title: String::new(),
            content: String::new(),
            topped: 0,
            reply_count: 0,
            created_at: SqlDateTime::MIN,
            refreshed_at: SqlDateTime::MIN,
        }
    }
}

pub async fn get_by_id(conn: &mut SqlConnection, id: u64) -> Result<TopicModel, ApiError> {
    let res = sqlx::query_as::<_, TopicModel>("select * from dg_topics where id=?")
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::TopicNotFound));
    }

    Ok(res.unwrap())
}

pub async fn create(conn: &mut SqlConnection, topic: &mut TopicModel) -> Result<u64, ApiError> {
    let res = sqlx::query(
        "insert into dg_topics(app_id,user_id,category,title,content,topped,refreshed_at) values(?,?,?,?,?,0,NOW())",
    )
    .bind(topic.app_id)
    .bind(topic.user_id)
    .bind(topic.category)
    .bind(&topic.title)
    .bind(&topic.content)
    .execute(conn)
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res.last_insert_id())
}

pub async fn update_reply_count(conn: &mut SqlConnection, id: u64) -> Result<(), ApiError> {
    sqlx::query("update dg_topics set reply_count=reply_count+1,refreshed_at=NOW() where id=?")
        .bind(id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}

pub async fn fetch_visibles(
    conn: &mut SqlConnection,
    app_id: u64,
    category: u64,
    order_by: VisiblesOrderBy,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<TopicModel>), ApiError> {
    let mut fetch_sql = String::new();
    let mut count_sql = String::new();
    let mut part_binds = Vec::new();

    fetch_sql.push_str("select * from dg_topics where app_id=?");
    count_sql.push_str("select count(*) from dg_topics where app_id=?");

    if category > 0 {
        fetch_sql.push_str(" and category=?");
        part_binds.push(category);
    }

    fetch_sql.push_str(" and topped>=0 order by topped desc");
    count_sql.push_str(" and topped>=0");

    if order_by != VisiblesOrderBy::REFRESH {
        fetch_sql.push_str(",created_at desc");
    } else {
        fetch_sql.push_str(",refreshed_at desc");
    }

    fetch_sql.push_str(" limit ?,?");

    let mut query = sqlx::query_as::<_, TopicModel>(&fetch_sql).bind(app_id);
    for v in part_binds.iter() {
        query = query.bind(v);
    }
    let topics = query
        .bind(cursor)
        .bind(count)
        .fetch_all(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let mut query = sqlx::query_as(&count_sql).bind(app_id);
    for v in part_binds.iter() {
        query = query.bind(v);
    }
    let total: (i64,) = query
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok((total.0 as u32, topics))
}
