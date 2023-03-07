use api_aga_in::database::*;
use api_aga_in::startup::run;
use std::sync::Arc;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").unwrap();
    let storage = Arc::new(storage(false).await);

    run(listener, storage).await
}
