use crate::configuration::{get_configuration, Settings};
use crate::database::*;
use crate::email_client::EmailClient;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<AppState> {
    use crate::routes::*;
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
        .route("/subscriptions/confirm", get(confirm))
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
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
}

pub fn run(
    listener: std::net::TcpListener,
    database: Arc<Database>,
    email_client: Arc<EmailClient>,
    base_url: String,
) -> impl std::future::Future<Output = hyper::Result<()>> {
    let _configuration = get_configuration();

    let app_state = AppState {
        database,
        email_client,
        base_url,
    };

    let app = router().with_state(app_state);

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}

pub struct Application {
    port: u16,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = hyper::Result<()>> + Send>>,
    database: Arc<Database>,
    host: String,
}

impl Application {
    pub async fn build(configuration: &Settings) -> Self {
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = Arc::new(EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            timeout,
        ));

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = std::net::TcpListener::bind(&address).unwrap();
        tracing::info!("Listening on {}", address);
        let host = configuration.application.host.clone();
        let port = listener.local_addr().unwrap().port();

        let storage = Arc::new(
            storage(
                &configuration.database.dir,
                configuration.database.memory_only,
            )
            .await,
        );
        let database = Arc::new(Database::init(storage.clone()).await);
        let server = Box::pin(run(
            listener,
            database.clone(),
            email_client,
            configuration.application.base_url.clone(),
        ));

        Self {
            server,
            database: database.clone(),
            port,
            host,
        }
    }

    // needs to consume to produce 1 server max, and because I don't know better
    pub fn server(self) -> impl std::future::Future<Output = hyper::Result<()>> + Send {
        self.server
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}
