use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    
    let address = format!("127.0.0.1:{}", config.application_port);
    let tcplistener = std::net::TcpListener::bind(address)
        .expect("Failed to bind port");
    run(tcplistener, db_pool).await
}
