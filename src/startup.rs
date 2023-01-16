use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{routes::*, telemetry::TowerMakeSpanWithConstantId};
use crate::middleware::RequestIdLayer;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub async fn run(listener: TcpListener, db_pool: PgPool) -> hyper::Result<()> {
    let address = listener.local_addr().expect("Failed to get local address");
    let app = app_router(db_pool);
    tracing::info!("listening on {}", address);
    // launch the application
    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await
}

pub fn app_router(db_pool: PgPool) -> Router {
    let app_state = AppState { 
        db_pool 
    };
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(app_state)
        // A span is created for each request and ends with the response is sent
        .layer(TraceLayer::new_for_http()
               .make_span_with(TowerMakeSpanWithConstantId)
        )
        .layer(RequestIdLayer)
}
