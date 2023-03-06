use crate::routes::*;
use axum::{
    routing::{get, post},
    Router, Server,
};

pub fn run(
    listener: std::net::TcpListener,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    #[rustfmt::skip]
    let app =
        Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
    ;

    println!("listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
