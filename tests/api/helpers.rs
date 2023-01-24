use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{Application, get_connection_pool};

pub struct TestApp {
    pub address: String,
    pub db_pool: sqlx::PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

// Ensure that the tracing stack is only initialized once using once_cell
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = String::from("info");
    let subscriber_name = String::from("test");

    // If you want to see all the logs set TEST_LOG to true ex:
    // TEST_LOG=true cargo #[test]
    // to make them pretty use: TEST_LOG=true cargo test health_check_works | bunyan
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// Launch application in the background
pub async fn spawn_app() -> TestApp {
    // Initialize tracing stack only once!
    Lazy::force(&TRACING);

    let config = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&config.database).await;

    let application = Application::build(config.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.port);
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp { address, db_pool: get_connection_pool(&config.database) }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect_with(&config.without_db())
            .await
            .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");
    db_pool
}
