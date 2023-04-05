use axum::{routing::get, Router};

mod app;
mod base;
mod manager;
mod reply;
mod topic;
mod user;

async fn root() -> &'static str {
    "DoggTalk MGR API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/manager", manager::setup_routers())
        .nest("/app", app::setup_routers())
        .nest("/user", user::setup_routers())
        .nest("/topic", topic::setup_routers())
        .nest("/reply", reply::setup_routers())
}
