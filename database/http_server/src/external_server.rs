use axum::{http::StatusCode, routing::get, Router};

pub async fn serve() -> hyper::Result<()> {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/cert", get(certificate));

    let addr = "0.0.0.0:4000";

    let listener = std::net::TcpListener::bind(addr).unwrap();

    println!("http server listening on {addr}");

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[axum_macros::debug_handler]
async fn certificate() -> Result<Vec<u8>, StatusCode> {
    // let q = "server-data.bonsaidb";
    match std::fs::read("server-data.bonsaidb/pinned-certificate.der") {
        Err(_e) => Err(StatusCode::NOT_FOUND),
        Ok(data) => Ok(data),
    }
}
