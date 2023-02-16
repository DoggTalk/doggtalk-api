use axum::{routing::get, Router};

mod manager;

async fn root() -> &'static str {
    "DoggTalk MGR API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/manager", manager::setup_routers())
}
