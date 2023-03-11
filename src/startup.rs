use crate::{configuration::get_configuration, database::Database};
use axum::{
    routing::{get, post},
    Router, Server,
};
use std::sync::Arc;

pub fn router() -> Router<AppState> {
    use crate::routes::*;
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_request_id::RequestIdLayer)
                .layer(
                    tower_http::trace::TraceLayer::new_for_http().make_span_with(
                        |request: &hyper::http::Request<hyper::Body>| {
                            // We get the request id from the extensions
                            let request_id = request
                                .extensions()
                                .get::<tower_request_id::RequestId>()
                                .map(ToString::to_string)
                                // .unwrap_or_else(|| "unknown".into());
                                .expect("Request ID assigning layer is missing");
                            // And then we put it along with other information into the `request` span
                            tracing::error_span!(
                                "request",
                                id = %request_id,
                                method = %request.method(),
                                uri = %request.uri(),
                            )
                        },
                    ),
                ),
        )
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

    Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
