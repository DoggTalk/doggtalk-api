use crate::shared::base::*;
use crate::shared::data::*;
use crate::shared::web::*;

const STATUS_HIDDEN: i64 = -1;
const STATUS_DELETE: i64 = -2;

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisibleStyle {
    ALL = 0,
    NORMAL = 1,
}

#[derive(PartialEq, Eq, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StatusAction {
    RESET = 0,
    MOVEUP = 1,
    HIDDEN = 2,
    DELETE = 3,
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
    pub like_count: u64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplySimple {
    pub id: u64,
    pub user_id: u64,
    pub content: String,
    pub topped: i64,
    pub created_at: SqlDateTime,
    pub like_count: u64,
}

impl ReplyModel {
    pub fn to_simple(self: &Self) -> ReplySimple {
        ReplySimple {
            id: self.id,
            user_id: self.user_id,
            content: self.content.clone(),
            topped: self.topped,
            created_at: self.created_at,
            like_count: self.like_count,
        }
    }

    pub fn is_actived(self: &Self) -> bool {
        return self.topped >= 0;
    }

    pub fn is_deleted(self: &Self) -> bool {
        return self.topped <= STATUS_DELETE;
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
            like_count: 0,
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

pub async fn update_status(
    conn: &mut SqlConnection,
    id: u64,
    action: StatusAction,
) -> Result<(), ApiError> {
    let mut topped: i64 = 0;
    if action == StatusAction::MOVEUP {
        topped = timestamp();
    } else if action == StatusAction::HIDDEN {
        topped = STATUS_HIDDEN;
    } else if action == StatusAction::DELETE {
        topped = STATUS_DELETE;
    }

    sqlx::query("update dg_replies set topped=? where id=?")
        .bind(topped)
        .bind(id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}

pub async fn update_like_count(
    conn: &mut SqlConnection,
    id: u64,
    op: UpdateCountOp,
) -> Result<(), ApiError> {
    let mut sql = String::new();
    sql.push_str("update dg_replies set like_count=like_count");

    let part_sql = match op {
        UpdateCountOp::INCR => "+1",
        _ => "-1",
    };
    sql.push_str(part_sql);
    sql.push_str(" where id=?");

    sqlx::query(&sql)
        .bind(id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
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

    let part_sql = match style {
        VisibleStyle::NORMAL => " and topped>=0",
        _ => " and topped>-2",
    };
    fetch_sql.push_str(part_sql);
    count_sql.push_str(part_sql);

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
