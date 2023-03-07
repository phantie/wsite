use crate::configuration::get_configuration;
use crate::routes::*;
use axum::{
    routing::{get, post},
    Router, Server,
};

use std::sync::Arc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<bonsaidb::local::AsyncStorage>,
}

pub fn run(
    listener: std::net::TcpListener,
    storage: Arc<bonsaidb::local::AsyncStorage>,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    let configuration = get_configuration();

    let app_state = AppState { storage };

    let app = router().with_state(app_state);

    println!("listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
