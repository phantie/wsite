use crate::routes::*;
use crate::{configuration::get_configuration, database::Database};
use axum::{
    routing::{get, post},
    Router, Server,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use std::sync::Arc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
}

pub fn run(
    listener: std::net::TcpListener,
    database: Arc<Database>,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    let _configuration = get_configuration();

    let app_state = AppState { database };

    let app = router().with_state(app_state);

    println!("listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
