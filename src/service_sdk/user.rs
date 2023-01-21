use axum::{routing::get, Router};

async fn root() -> &'static str {
    "DoggTalk SDK User API"
}

pub fn setup_routers() -> Router {
    Router::new().route("/", get(root))
}
