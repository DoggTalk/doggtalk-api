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

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisibleOrderBy {
    CREATE = 0,
    REFRESH = 1,
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
pub struct TopicModel {
    pub id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub category: u64,
    pub title: String,
    pub content: String,
    pub topped: i64,
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
    pub topped: i64,
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

    pub fn is_actived(self: &Self) -> bool {
        return self.topped >= 0;
    }

    pub fn is_deleted(self: &Self) -> bool {
        return self.topped <= STATUS_DELETE;
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

pub async fn update_reply_count(
    conn: &mut SqlConnection,
    id: u64,
    op: UpdateCountOp,
) -> Result<(), ApiError> {
    let mut sql = String::new();
    sql.push_str("update dg_topics set reply_count=reply_count");

    let part_sql = match op {
        UpdateCountOp::INCR => "+1,refreshed_at=NOW()",
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

    sqlx::query("update dg_topics set topped=? where id=?")
        .bind(topped)
        .bind(id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}

pub async fn fetch_more(
    conn: &mut SqlConnection,
    app_id: u64,
    category: u64,
    style: VisibleStyle,
    order_by: VisibleOrderBy,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<TopicModel>), ApiError> {
    let mut fetch_sql = String::new();
    let mut count_sql = String::new();
    let mut part_binds = Vec::new();

    fetch_sql.push_str("select * from dg_topics where app_id=?");
    count_sql.push_str("select count(*) from dg_topics where app_id=?");
    part_binds.push(app_id);

    if category > 0 {
        let part_sql = " and category=?";
        fetch_sql.push_str(part_sql);
        count_sql.push_str(part_sql);
        part_binds.push(category);
    }

    let part_sql = match style {
        VisibleStyle::NORMAL => " and topped>=0",
        _ => " and topped>-2",
    };
    fetch_sql.push_str(part_sql);
    count_sql.push_str(part_sql);

    fetch_sql.push_str(" order by ");
    let part_sql = match order_by {
        VisibleOrderBy::REFRESH => "created_at desc",
        _ => "topped desc,refreshed_at desc",
    };
    fetch_sql.push_str(part_sql);
    fetch_sql.push_str(" limit ?,?");

    let mut query = sqlx::query_as::<_, TopicModel>(&fetch_sql);
    for v in part_binds.iter() {
        query = query.bind(v);
    }
    let topics = query
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

    Ok((total.0 as u32, topics))
}
