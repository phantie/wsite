use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::startup::run;
use std::sync::Arc;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let configuration = get_configuration();
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").unwrap();
    let storage = Arc::new(storage(&configuration.database.dir, false).await);
    let database = Arc::new(Database::init(storage.clone()).await);
    run(listener, database).await
}
