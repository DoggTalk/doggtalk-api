use axum::{routing::get, Router};

async fn root() -> &'static str {
    "DoggTalk MGR API"
}

pub fn setup_routers() -> Router {
    Router::new().route("/", get(root))
}
