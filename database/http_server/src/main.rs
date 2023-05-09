use axum::{
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tower_http::add_extension::AddExtensionLayer;
mod database;

struct HostedDatabase {
    handle: JoinHandle<Result<(), bonsaidb::server::Error>>,
}

impl HostedDatabase {
    fn running(&self) -> bool {
        !self.handle.is_finished()
    }
}

type SharedHostedDatabase = Arc<HostedDatabase>;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let database_server = database::server().await.unwrap();

    let handle = tokio::spawn(async move {
        println!("database server is listening on 5645");
        database_server.clone().listen_on(5645).await
    });

    let hosted_database = Arc::new(HostedDatabase { handle });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/database/info", get(database_info))
        .layer(AddExtensionLayer::new(hosted_database));

    let addr = database_common::ADDR;

    let listener = std::net::TcpListener::bind(addr).unwrap();

    println!("http server listening on {addr}");

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[axum_macros::debug_handler]
async fn database_info(
    Extension(database_server): Extension<SharedHostedDatabase>,
) -> Json<interfacing::DatabaseInfo> {
    Json::from(interfacing::DatabaseInfo {
        is_running: database_server.running(),
    })
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
