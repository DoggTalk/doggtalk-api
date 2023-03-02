use crate::shared::base::timestamp;
use crate::shared::data::*;
use crate::shared::web::*;

use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppModel {
    pub id: u64,
    pub app_key: String,
    pub app_secret: String,
    pub name: String,
    pub icon_url: String,
    pub created_at: SqlDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppSimple {
    pub id: u64,
    pub app_key: String,
    pub name: String,
    pub icon_url: String,
}

pub fn build_key() -> String {
    let ts = timestamp();
    let r = rand::random::<u16>();

    let mut out = String::new();
    out.push_str(&base62::encode(r));
    out.push_str(&base62::encode(ts));
    out
}

pub async fn get_by_id(mut conn: SqlConnection, id: u64) -> Result<AppModel, ApiError> {
    let res = sqlx::query_as::<_, AppModel>("select * from dg_app where id=?")
        .bind(id)
        .fetch_optional(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AppNotFound));
    }

    Ok(res.unwrap())
}

pub async fn create(mut conn: SqlConnection, app: AppModel) -> Result<u64, ApiError> {
    let res = sqlx::query("insert into dg_app(app_key,app_secret,name,icon_url) values(?,?,?,?)")
        .bind(app.app_key)
        .bind(app.app_secret)
        .bind(app.name)
        .bind(app.icon_url)
        .execute(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res.last_insert_id())
}

pub async fn fetch_more(
    mut conn: SqlConnection,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<AppModel>), ApiError> {
    let res = sqlx::query_as::<_, AppModel>("select * from dg_app order by id desc limit ?,?")
        .bind(cursor)
        .bind(count)
        .fetch_all(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let total: (i64,) = sqlx::query_as("select count(*) from dg_app")
        .fetch_one(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok((total.0 as u32, res))
}

pub async fn fetch_simple_all(mut conn: SqlConnection) -> Result<Vec<AppSimple>, ApiError> {
    let res =
        sqlx::query_as::<_, AppSimple>("select id,app_key,name,icon_url from dg_app order by id")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res)
}
