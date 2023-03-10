use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::shared::data::*;
use crate::shared::web::*;

pub const SOURCE_FAKE: i8 = 0;
pub const SOURCE_SYNC: i8 = 1;
pub const STATUS_PENDING: i8 = 0;
pub const STATUS_ACTIVED: i8 = 1;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserModel {
    pub id: u64,
    pub app_id: u64,
    pub source: i8,
    pub account: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub gender: i8,
    pub status: i8,
    pub created_at: SqlDateTime,
    pub topic_count: u64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSimple {
    pub id: u64,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: i8,
    pub gender: i8,
}

impl UserModel {
    pub fn to_simple(self: &Self) -> UserSimple {
        UserSimple {
            id: self.id,
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone(),
            status: self.status,
            gender: self.gender,
        }
    }

    pub fn is_actived(self: &Self) -> bool {
        return self.status >= STATUS_ACTIVED;
    }
}

impl Default for UserModel {
    fn default() -> UserModel {
        UserModel {
            id: 0,
            app_id: 0,
            source: SOURCE_SYNC,
            account: String::new(),
            display_name: String::new(),
            avatar_url: None,
            status: STATUS_ACTIVED,
            gender: 0,
            created_at: SqlDateTime::MIN,
            topic_count: 0,
        }
    }
}

impl Default for UserSimple {
    fn default() -> UserSimple {
        UserSimple {
            id: 0,
            display_name: String::new(),
            avatar_url: None,
            status: STATUS_PENDING,
            gender: 0,
        }
    }
}

pub type ArcUserSimple = Arc<UserSimple>;
pub static DEFAULT_SIMPLE: Lazy<ArcUserSimple> = Lazy::new(|| Arc::new(Default::default()));

pub async fn get_by_id(conn: &mut SqlConnection, id: u64) -> Result<UserModel, ApiError> {
    let res = sqlx::query_as::<_, UserModel>("select * from dg_users where id=?")
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AccountNotFound));
    }

    Ok(res.unwrap())
}

pub async fn get_by_account(
    conn: &mut SqlConnection,
    app_id: u64,
    source: u16,
    account: &str,
) -> Result<Option<UserModel>, ApiError> {
    let res = sqlx::query_as::<_, UserModel>(
        "select * from dg_users where app_id=? and source=? and account=?",
    )
    .bind(app_id)
    .bind(source)
    .bind(account)
    .fetch_optional(conn)
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res)
}

pub async fn get_simple_map_by_ids(
    conn: &mut SqlConnection,
    ids: Vec<u64>,
) -> Result<HashMap<u64, ArcUserSimple>, ApiError> {
    if ids.len() < 1 {
        return Ok(HashMap::new());
    }

    let ids_str = ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");
    let res = sqlx::query_as::<_, UserSimple>(&format!(
        "select id,display_name,avatar_url,status,gender from dg_users where id in ({})",
        ids_str
    ))
    .fetch_all(conn)
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let mut out = HashMap::new();
    for o in res {
        out.insert(o.id, Arc::new(o));
    }

    Ok(out)
}

pub async fn create(conn: &mut SqlConnection, user: &mut UserModel) -> Result<u64, ApiError> {
    let res = sqlx::query(
        "insert into dg_users(app_id,source,account,display_name,avatar_url,gender,status) values(?,?,?,?,?,?,?)",
    )
    .bind(user.app_id)
    .bind(user.source)
    .bind(&user.account)
    .bind(&user.display_name)
    .bind(&user.avatar_url)
    .bind(user.gender)
    .bind(user.status)
    .execute(conn)
    .await
    .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res.last_insert_id())
}

pub async fn update_profile(
    conn: &mut SqlConnection,
    user: &mut UserModel,
) -> Result<(), ApiError> {
    sqlx::query("update dg_users set gender=?,display_name=?,avatar_url=? where id=?")
        .bind(user.gender)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}

pub async fn update_topic_count(conn: &mut SqlConnection, id: u64) -> Result<(), ApiError> {
    sqlx::query("update dg_users set topic_count=topic_count+1 where id=?")
        .bind(id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}
