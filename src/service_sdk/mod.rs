use axum::{routing::get, Router};

mod home;
mod user;

async fn root() -> &'static str {
    "DoggTalk SDK API"
}

pub fn setup_routers() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/user", user::setup_routers())
        .nest("/home", home::setup_routers())
}
