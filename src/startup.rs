use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::net::TcpListener;
use crate::routes::*;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub async fn run(listener: TcpListener, db_pool: PgPool) -> hyper::Result<()> {
    let app = app_router(db_pool);
    // launch the application
    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await
}

pub fn app_router(db_pool: PgPool) -> Router {
    let app_state = AppState { 
        db_pool 
    };
    // build our application with a single route
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
            .with_state(app_state)
}
