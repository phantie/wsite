use crate::configuration::get_configuration;
use crate::database::UserDatabase;
use crate::routes::*;
use axum::{
    routing::{get, post},
    Router, Server,
};

// #[derive(Clone, Default)]
// pub struct AppState {}

pub fn run(
    listener: std::net::TcpListener,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    let configuration = get_configuration().unwrap();

    dbg!(configuration);

    #[rustfmt::skip]
    let app =
        Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        // .with_state(AppState::default())
    ;

    println!("listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
