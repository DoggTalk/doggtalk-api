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

impl Default for ManagerModel {
    fn default() -> ManagerModel {
        ManagerModel {
            id: 0,
            username: String::new(),
            password: String::new(),
            created_at: SqlDateTime::MIN,
        }
    }
}

pub async fn get_by_username(
    conn: &mut SqlConnection,
    username: &str,
) -> Result<Option<ManagerModel>, ApiError> {
    let res = sqlx::query_as::<_, ManagerModel>("select * from dg_managers where username=?")
        .bind(username)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    Ok(res)
}

pub async fn get_by_id(conn: &mut SqlConnection, id: u64) -> Result<ManagerModel, ApiError> {
    let res = sqlx::query_as::<_, ManagerModel>("select * from dg_managers where id=?")
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))?;

    if res.is_none() {
        return Err(api_error(ApiErrorCode::AccountNotFound));
    }

    Ok(res.unwrap())
}
