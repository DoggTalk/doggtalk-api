use axum::{routing::get, Router};

mod app;
mod base;
mod manager;

async fn root() -> &'static str {
    "DoggTalk MGR API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/manager", manager::setup_routers())
        .nest("/app", app::setup_routers())
}
