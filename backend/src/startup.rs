use crate::configuration::Settings;
use crate::database::*;
use crate::email_client::EmailClient;
use axum::{
    routing::{get, post},
    Router,
};
use axum_sessions::{
    async_session::{async_trait, Session, SessionStore},
    SessionLayer,
};
use bonsaidb::core::keyvalue::AsyncKeyValue;
use secrecy::ExposeSecret;
use std::sync::Arc;

pub fn router(sessions: Arc<Database>) -> Router<AppState> {
    use crate::routes::*;
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions", get(all_subscriptions))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .route("/", get(home))
        .route("/login", get(login_form))
        .route("/login", post(login))
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/password", get(change_password_form))
        .route("/admin/password", post(change_password))
        .route("/admin/logout", post(logout))
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
        .layer({
            // let store = axum_sessions::async_session::MemoryStore::new();
            let store = BonsaiDBSessionStore { database: sessions };
            // FIXIT make it persistent and secret
            let secret = [0_u8; 128];

            // use rand::Rng;
            // let mut secret = [0_u8; 128];
            // rand::thread_rng().fill(&mut secret);

            SessionLayer::new(store, &secret).with_secure(true)
        })
}

#[derive(Clone, Debug)]
struct BonsaiDBSessionStore {
    database: Arc<Database>,
}

#[async_trait]
impl SessionStore for BonsaiDBSessionStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> axum_sessions::async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let session: Option<Session> = self.database.sessions.get_key(id).into().await?;

        Ok(session.and_then(Session::validate))
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> axum_sessions::async_session::Result<Option<String>> {
        self.database
            .sessions
            .set_key(session.id().to_string(), &session)
            .await?;
        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> axum_sessions::async_session::Result {
        self.database
            .sessions
            .delete_key(session.id().to_string())
            .await?;
        Ok(())
    }

    async fn clear_store(&self) -> axum_sessions::async_session::Result {
        tracing::info!("DELETE ALL SESSIONS");
        todo!("find out how to delete all keys from the storage")
        // self.database.sessions.
        // Ok(())
    }
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
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
        tracing::info!("Listening on http://{}", address);
        let host = configuration.application.host.clone();
        let port = listener.local_addr().unwrap().port();

        {
            let first_symbols_count = 3;
            let first_symbols_of_email_token = &(*configuration
                .email_client
                .authorization_token
                .expose_secret())[..first_symbols_count];
            tracing::info!(
                "First symbols of email token:{}",
                first_symbols_of_email_token
            );
        }

        let storage = Arc::new(
            storage(
                &configuration.database.dir,
                configuration.database.memory_only,
            )
            .await,
        );
        let database = Arc::new(Database::init(storage.clone()).await);

        pub fn run(
            listener: std::net::TcpListener,
            database: Arc<Database>,
            email_client: Arc<EmailClient>,
            base_url: String,
        ) -> impl std::future::Future<Output = hyper::Result<()>> {
            let app_state = AppState {
                database,
                email_client,
                base_url,
            };

            let app = router(app_state.database.clone()).with_state(app_state);

            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
        }

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
