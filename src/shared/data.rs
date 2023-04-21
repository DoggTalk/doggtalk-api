use std::time::Duration;

use bb8_redis::{bb8, RedisConnectionManager};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::shared::base::*;
use crate::shared::web::*;

#[derive(PartialEq, Eq)]
pub enum UpdateCountOp {
    INCR,
    DECR,
}

pub fn init() {
    Lazy::force(&MYSQL_POOL);
    Lazy::force(&REDIS_POOL);
}

pub type SqlDateTime = chrono::NaiveDateTime;
pub type SqlConnection = sqlx::pool::PoolConnection<sqlx::MySql>;

static MYSQL_POOL: Lazy<MySqlPool> = Lazy::new(|| {
    let connection_str = std::env::var("MYSQL_URL").expect("MYSQL_URL must be set");
    let max_connections: u32 = std::env::var("MYSQL_MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse()
        .expect("MYSQL_MAX_CONNECTIONS must an int");

    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy(&connection_str)
        .expect("can't connect to database")
});

pub async fn database_connect() -> Result<SqlConnection, ApiError> {
    MYSQL_POOL
        .acquire()
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))
}

pub use redis::aio::ConnectionLike as RedisConnectionLike;
pub type RedisConnection = bb8::PooledConnection<'static, RedisConnectionManager>;

static REDIS_POOL: Lazy<bb8::Pool<RedisConnectionManager>> = Lazy::new(|| {
    let connection_str = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let max_connections: u32 = std::env::var("REDIS_MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse()
        .expect("REDIS_MAX_CONNECTIONS must an int");

    let manager = RedisConnectionManager::new(connection_str).unwrap();

    bb8::Pool::builder()
        .max_size(max_connections)
        .build_unchecked(manager)
});

pub async fn redis_connect() -> Result<RedisConnection, ApiError> {
    REDIS_POOL
        .get()
        .await
        .map_err(|e| api_errore(ApiErrorCode::InvalidDatabase, &e))
}
