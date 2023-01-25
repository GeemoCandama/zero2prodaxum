use axum::{
    body::Body,
    routing::{get, post, IntoMakeService},
    Router,
};
use hyper::server::{conn::AddrIncoming, Server};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::middleware::RequestIdLayer;
use crate::{routes::*, telemetry::TowerMakeSpanWithConstantId};

pub struct Application 
{
    pub port: u16,
    pub server: MyServer,
}

impl Application 
{
    pub async fn build(config: Settings) -> hyper::Result<Self> {
        let db_pool = get_connection_pool(&config.database);

        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = config.email_client.timeout();
        let email_client = EmailClient::new(
            sender_email,
            config.email_client.base_url,
            config.email_client.authorization_token,
            timeout,
        );

        let address = format!("{}:{}", config.application.host, config.application.port);
        let tcplistener = std::net::TcpListener::bind(address).expect("Failed to bind port");
        let port = tcplistener.local_addr().unwrap().port();
        let server = run(tcplistener, db_pool, email_client, config.application.base_url)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> hyper::Result<()> {
        self.server.await
    }
}

pub type MyServer = Server<AddrIncoming, IntoMakeService<Router<(), Body>>>;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub email_client: EmailClient,
    pub base_url: String,
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<MyServer, hyper::Error> {
    let address = listener.local_addr().expect("Failed to get local address");
    let app = app_router(db_pool, email_client, base_url);
    tracing::info!("listening on {}", address);
    // launch the application
    let server = axum::Server::from_tcp(listener)?;
    Ok(server.serve(app.into_make_service()))
}

pub fn app_router(db_pool: PgPool, email_client: EmailClient, base_url: String) -> Router {
    let app_state = AppState {
        db_pool,
        email_client,
        base_url,
    };
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .with_state(app_state)
        // A span is created for each request and ends with the response is sent
        .layer(TraceLayer::new_for_http().make_span_with(TowerMakeSpanWithConstantId))
        .layer(RequestIdLayer)
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}
