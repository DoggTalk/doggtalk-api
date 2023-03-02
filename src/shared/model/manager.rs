use crate::shared::data::*;
use crate::shared::web::*;

use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManagerModel {
    pub id: u64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: SqlDateTime,
}

pub async fn get_by_username(
    mut conn: SqlConnection,
    username: &str,
) -> Result<Option<ManagerModel>, ApiError> {
    let res = sqlx::query_as::<_, ManagerModel>("select * from dg_manager where username=?")
        .bind(username)
        .fetch_optional(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res)
}

pub async fn get_by_id(mut conn: SqlConnection, id: u64) -> Result<ManagerModel, ApiError> {
    let res = sqlx::query_as::<_, ManagerModel>("select * from dg_manager where id=?")
        .bind(id)
        .fetch_optional(&mut conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AccountNotFound));
    }

    Ok(res.unwrap())
}
