use once_cell::sync::Lazy;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;

use crate::shared::web::*;

static MYSQL_POOL: Lazy<MySqlPool> = Lazy::new(|| {
    let connection_str = std::env::var("MYSQL_URL").expect("MYSQL_URL must be set");
    let max_connections: u32 = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or("5".to_string())
        .parse()
        .expect("DB_MAX_CONNECTIONS must an int");

    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy(&connection_str)
        .expect("can't connect to database")
});

pub fn init() {
    Lazy::force(&MYSQL_POOL);
}

pub type SqlDateTime = chrono::NaiveDateTime;
pub type SqlConnection = sqlx::pool::PoolConnection<sqlx::MySql>;

pub async fn database_connect() -> Result<SqlConnection, ApiError> {
    MYSQL_POOL
        .acquire()
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))
}
