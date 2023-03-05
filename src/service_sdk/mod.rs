use axum::{routing::get, Router};

mod base;
mod home;
mod reply;
mod topic;
mod user;

async fn root() -> &'static str {
    "DoggTalk SDK API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/home", home::setup_routers())
        .nest("/user", user::setup_routers())
        .nest("/topic", topic::setup_routers())
        .nest("/reply", reply::setup_routers())
}
