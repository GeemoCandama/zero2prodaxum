#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch application in the background
fn spawn_app() -> String {
    use std::net::{TcpListener};

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(zero2prod::app_router().into_make_service())
            .await
            .unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}
