use std::net::TcpListener;

use axum::{
    routing::get,
    Router,
};

async fn health_check() {}

pub fn app_router() -> Router {
    // build our application with a single route
    Router::new()
        .route("/health_check", get(health_check))
}

pub async fn run(listener: TcpListener) -> hyper::Result<()> {
    let app = app_router();
    // launch the application
    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await
}
