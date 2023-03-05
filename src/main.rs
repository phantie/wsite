use api_aga_in::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").unwrap();

    run(listener).await
}
