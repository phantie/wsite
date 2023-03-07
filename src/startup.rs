use crate::configuration::get_configuration;
use crate::routes::*;
use axum::{
    routing::{get, post},
    Router, Server,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
}

#[derive(Clone, Default)]
pub struct AppState {}

pub fn run(
    listener: std::net::TcpListener,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    let _configuration = get_configuration().unwrap();

    let app_state = AppState::default();

    let app = router().with_state(app_state);

    println!("listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
