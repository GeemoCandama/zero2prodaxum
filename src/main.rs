use zero2prod::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let tcplistener = std::net::TcpListener::bind("127.0.0.1:8000")
        .expect("Failed to bind port");
    run(tcplistener).await
}
