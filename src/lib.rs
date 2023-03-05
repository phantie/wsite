use axum::{http::StatusCode, response::Html, routing::get, Router};
use std::future::Future;
use std::net::SocketAddr;

pub fn run() -> impl Future<Output = hyper::Result<()>> {
    #[rustfmt::skip]
    let app =
        Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
    ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service())
}

async fn index() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
