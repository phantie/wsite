use axum::{
    extract::{Form, Json},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::future::Future;

pub fn run(listener: std::net::TcpListener) -> impl Future<Output = hyper::Result<()>> {
    #[rustfmt::skip]
    let app =
        Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
    ;

    println!("listening on {}", listener.local_addr().unwrap());
    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}

async fn index() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct SubscribeUser {
    email: String,
    name: String,
}

async fn subscribe(Form(payload): Form<SubscribeUser>) -> StatusCode {
    StatusCode::OK
}
