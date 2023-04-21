use axum::{http::StatusCode, routing::get, Router};
use std::net::SocketAddr;

mod service_mgr;
mod service_sdk;
mod shared;

#[tokio::main]
async fn main() {
    let web_port: u16 = std::env::var("WEB_PORT")
        .unwrap_or("6000".to_string())
        .parse()
        .expect("WEB_PORT must an int");

    shared::init();

    let app = Router::new()
        .route("/", get(root))
        .nest("/sdk", service_sdk::setup_routers())
        .nest("/mgr", service_mgr::setup_routers())
        .fallback(fallback);

    let addr = SocketAddr::from(([0, 0, 0, 0], web_port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "DoggTalk API"
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
