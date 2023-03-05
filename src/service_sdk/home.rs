use axum::{extract::Query, routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

pub fn setup_routers() -> Router {
    Router::new().route("/", get(app_home))
}

#[derive(Deserialize)]
struct AppHomePayload {
    app_key: String,
}

#[derive(Serialize)]
struct AppHomeResponse {
    app: app::AppSimple,
}

async fn app_home(
    Query(payload): Query<AppHomePayload>,
) -> Result<ApiSuccess<AppHomeResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let app = app::get_by_key(&mut conn, &payload.app_key).await?;

    Ok(api_success(AppHomeResponse {
        app: app.to_simple(),
    }))
}
