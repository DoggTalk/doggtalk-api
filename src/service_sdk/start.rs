use axum::{routing::get, Router};

use crate::shared::data::*;
use crate::shared::model::*;
use crate::shared::web::*;

pub fn setup_routers() -> Router {
    Router::new().route("/", get(app_start))
}

#[derive(Deserialize)]
struct AppStartPayload {
    app_key: String,
}

#[derive(Serialize)]
struct AppStartResponse {
    app: app::AppSimple,
}

async fn app_start(
    Query(payload): Query<AppStartPayload>,
) -> Result<ApiSuccess<AppStartResponse>, ApiError> {
    let mut conn = database_connect().await?;

    let app = app::get_by_key(&mut conn, &payload.app_key).await?;

    Ok(api_success(AppStartResponse {
        app: app.to_simple(),
    }))
}
