use crate::shared::data::*;
use crate::shared::web::*;

use serde::Serialize;

pub const SOURCE_FAKE: i8 = 0;
pub const SOURCE_SYNC: i8 = 1;
pub const STATUS_ACTIVED: i8 = 1;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserModel {
    pub id: u64,
    pub app_id: u64,
    pub source: i8,
    pub account: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: i8,
    pub created_at: SqlDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSimple {
    pub id: u64,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl UserModel {
    pub fn to_simple(self: &Self) -> UserSimple {
        UserSimple {
            id: self.id,
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone(),
        }
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
            created_at: SqlDateTime::MIN,
        }
    }
}

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

pub async fn create(conn: &mut SqlConnection, user: &mut UserModel) -> Result<u64, ApiError> {
    let res = sqlx::query(
        "insert into dg_users(app_id,source,account,display_name,avatar_url,status) values(?,?,?,?,?,?)",
    )
    .bind(user.app_id)
    .bind(user.source)
    .bind(&user.account)
    .bind(&user.display_name)
    .bind(&user.avatar_url)
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
    sqlx::query("update dg_users set display_name=?,avatar_url=? where id=?")
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.id)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(())
}
