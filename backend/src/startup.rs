use crate::{configuration::Settings, database::*, email_client::EmailClient};
use static_routes::*;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_sessions::{
    async_session::{async_trait, Session, SessionStore},
    SessionLayer,
};
use bonsaidb::core::keyvalue::AsyncKeyValue;
use secrecy::ExposeSecret;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{
    add_extension::AddExtensionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

static FRONTEND_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist/");

static INDEX_HTML: &str = include_str!("../../frontend/dist/index.html");

async fn fallback(uri: axum::http::Uri) -> axum::response::Response {
    use axum::body::{self, Full};
    use axum::http::{header, HeaderValue, StatusCode};
    use axum::response::Html;
    use axum::response::IntoResponse;

    let path = uri.to_string();
    let path = path.trim_start_matches('/');

    match FRONTEND_DIR.get_file(path) {
        None => Html(INDEX_HTML).into_response(),
        Some(file) => {
            let mime_type = mime_guess::from_path(path).first_or_text_plain();

            axum::http::Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                )
                .body(body::boxed(Full::from(file.contents())))
                .unwrap()
        }
    }
}

#[derive(Clone, Default)]
pub struct RequestIdProducer {
    counter: Arc<std::sync::atomic::AtomicU64>,
}

impl tower_http::request_id::MakeRequestId for RequestIdProducer {
    fn make_request_id<B>(
        &mut self,
        _request: &hyper::http::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(tower_http::request_id::RequestId::new(request_id))
    }
}

pub fn router(sessions: Arc<Database>, shared_state: SharedRemoteDatabase) -> Router<AppState> {
    use crate::routes::*;

    let routes = routes().api;

    let api_router = Router::new()
        .route(routes.health_check.get().postfix(), get(health_check))
        .route(routes.subs.get().postfix(), get(all_subs))
        .route(routes.subs.new.post().postfix(), post(subscribe))
        .route(routes.subs.confirm.get().postfix(), get(sub_confirm))
        .route(
            routes.newsletters.post().postfix(),
            post(publish_newsletter),
        )
        .route(routes.login.post().postfix(), post(login))
        .route(
            routes.admin.password.post().postfix(),
            post(change_password),
        )
        .route(routes.admin.logout.post().postfix(), post(logout))
        .route(routes.admin.session.get().postfix(), get(admin_session))
        .route(routes.articles.get().postfix(), get(article_list))
        .route("/articles/:public_id", get(article_by_public_id))
        .route("/articles/:public_id", delete(delete_article))
        .route(routes.admin.articles.post().postfix(), post(new_article))
        .route("/admin/articles", put(update_article))
        .route("/shapes", get(all_shapes))
        .route("/shapes", post(new_shape));

    let request_tracing_layer = tower::ServiceBuilder::new()
        .set_x_request_id(RequestIdProducer::default())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::DEBUG).include_headers(true))
                .make_span_with(|request: &hyper::http::Request<hyper::Body>| {
                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        request_id = %request.headers().get("x-request-id").unwrap().to_str().unwrap(),
                    )
                })
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Seconds),
                ),
        )
        .propagate_x_request_id();

    Router::new()
        .nest("/api", api_router)
        .fallback(fallback)
        .layer(AddExtensionLayer::new(shared_state))
        .layer(request_tracing_layer)
        .layer({
            // let store = axum_sessions::async_session::MemoryStore::new();
            let store = BonsaiDBSessionStore { database: sessions };
            // FIXIT make it persistent and secret
            let secret = [0_u8; 128];

            // use rand::Rng;
            // let mut secret = [0_u8; 128];
            // rand::thread_rng().fill(&mut secret);

            SessionLayer::new(store, &secret).with_secure(false)
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

pub type SharedRemoteDatabase = Arc<RwLock<RemoteDatabase>>;

pub struct Application {
    port: u16,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = hyper::Result<()>> + Send>>,
    database: Arc<Database>,
    host: String,
    #[allow(dead_code)]
    shared_state: SharedRemoteDatabase,
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
            let first_symbols_count = 5;
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
            shared_state: SharedRemoteDatabase,
        ) -> impl std::future::Future<Output = hyper::Result<()>> {
            let app_state = AppState {
                database,
                email_client,
                base_url,
            };

            let app = router(app_state.database.clone(), shared_state).with_state(app_state);

            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
        }

        let remote_database = RemoteDatabase::configure(
            "abada-dabada",
            RemoteClientParams {
                url: "bonsaidb://209.38.192.88".into(),
                // url: "bonsaidb://165.22.74.247".into(),
                password: "1".into(),
            },
        )
        .await
        .expect("database must be available on deployment");

        let shared_state = Arc::new(RwLock::new(remote_database));

        let server = Box::pin(run(
            listener,
            database.clone(),
            email_client,
            configuration.application.base_url.clone(),
            shared_state.clone(),
        ));

        Self {
            server,
            database: database.clone(),
            port,
            host,
            shared_state,
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
