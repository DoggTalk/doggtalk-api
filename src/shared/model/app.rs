use crate::shared::base::timestamp;
use crate::shared::data::*;
use crate::shared::web::*;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppModel {
    pub id: u64,
    pub app_key: String,
    pub app_secret: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub created_at: SqlDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppSimple {
    pub id: u64,
    pub app_key: String,
    pub name: String,
    pub icon_url: Option<String>,
}

impl AppModel {
    pub fn to_simple(self: &Self) -> AppSimple {
        AppSimple {
            id: self.id,
            app_key: self.app_key.clone(),
            name: self.name.clone(),
            icon_url: self.icon_url.clone(),
        }
    }
}

impl Default for AppModel {
    fn default() -> AppModel {
        AppModel {
            id: 0,
            app_key: String::new(),
            app_secret: String::new(),
            name: String::new(),
            icon_url: None,
            created_at: SqlDateTime::MIN,
        }
    }
}

pub fn build_key() -> String {
    let ts = timestamp() as u64;
    let r = rand::random::<u16>();

    let mut out = String::new();
    out.push_str(&base62::encode(r));
    out.push_str(&base62::encode(ts));
    out
}

pub async fn get_by_id(conn: &mut SqlConnection, id: u64) -> Result<AppModel, ApiError> {
    let res = sqlx::query_as::<_, AppModel>("select * from dg_apps where id=?")
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AppNotFound));
    }

    Ok(res.unwrap())
}

pub async fn get_by_key(conn: &mut SqlConnection, app_key: &str) -> Result<AppModel, ApiError> {
    let res = sqlx::query_as::<_, AppModel>("select * from dg_apps where app_key=?")
        .bind(app_key)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AppNotFound));
    }

    Ok(res.unwrap())
}

pub async fn create(conn: &mut SqlConnection, app: AppModel) -> Result<u64, ApiError> {
    let res = sqlx::query("insert into dg_apps(app_key,app_secret,name,icon_url) values(?,?,?,?)")
        .bind(app.app_key)
        .bind(app.app_secret)
        .bind(app.name)
        .bind(app.icon_url)
        .execute(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res.last_insert_id())
}

pub async fn fetch_more(
    conn: &mut SqlConnection,
    cursor: u32,
    count: u32,
) -> Result<(u32, Vec<AppModel>), ApiError> {
    let res = sqlx::query_as::<_, AppModel>("select * from dg_apps order by id desc limit ?,?")
        .bind(cursor)
        .bind(count)
        .fetch_all(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    let total: (i64,) = sqlx::query_as("select count(*) from dg_apps")
        .fetch_one(conn.as_mut())
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok((total.0 as u32, res))
}
