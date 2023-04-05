use crate::shared::data::*;
use crate::shared::web::*;

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisibleStyle {
    ALL = 0,
    NORMAL = 1,
}

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

pub async fn fetch_more(
    conn: &mut SqlConnection,
    topic_id: u64,
    style: VisibleStyle,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<ReplyModel>), ApiError> {
    let mut fetch_sql = String::new();
    let mut count_sql = String::new();
    let mut part_binds = Vec::new();

    fetch_sql.push_str("select * from dg_replies where topic_id=?");
    count_sql.push_str("select count(*) from dg_replies where topic_id=?");
    part_binds.push(topic_id);

    if style == VisibleStyle::NORMAL {
        fetch_sql.push_str(" and topped>=0");
        count_sql.push_str(" and topped>=0");
    }

    fetch_sql.push_str(" order by created_at desc limit ?,?");

    let mut query = sqlx::query_as::<_, ReplyModel>(&fetch_sql);
    for v in part_binds.iter() {
        query = query.bind(v);
    }
    let replies = query
        .bind(cursor)
        .bind(count)
        .fetch_all(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let mut query = sqlx::query_as(&count_sql);
    for v in part_binds.iter() {
        query = query.bind(v);
    }
    let total: (i64,) = query
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok((total.0 as u32, replies))
}
