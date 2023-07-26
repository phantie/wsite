use crate::{
    configuration::{get_env, Conf},
    database::*,
    email_client::EmailClient,
    error::ApiResult,
    timeout::HangingStrategy,
};
use common::static_routes::*;

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
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

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

pub fn router(conf: &Conf, db_client: SharedDbClient) -> Router<AppState> {
    use crate::routes::*;

    let routes = routes().api;

    let api_router = Router::new()
        .route(routes.health_check.get().postfix(), get(health_check))
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
        .route("/static/:path", get(serve_static))
        .route("/users_online", get(ws_users_online));

    let api_router = if conf.env.features.newsletter {
        api_router
            .route(routes.subs.get().postfix(), get(all_subs))
            .route(routes.subs.new.post().postfix(), post(subscribe))
            .route(routes.subs.confirm.get().postfix(), get(sub_confirm))
            .route(
                routes.newsletters.post().postfix(),
                post(publish_newsletter),
            )
    } else {
        api_router
    };

    let api_router = if get_env().local() {
        api_router
            .route("/shapes", get(all_shapes))
            .route("/shapes", post(new_shape))
    } else {
        api_router
    };

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
        .layer(CompressionLayer::new())
        .layer(AddExtensionLayer::new(db_client.clone()))
        .layer(request_tracing_layer)
        .layer({
            // let store = axum_sessions::async_session::MemoryStore::new();
            let store = BonsaiDBSessionStore { db_client };

            let decoded = hex::decode(conf.env.session_secret.clone())
                .expect("HEX Decoding of session secret failed");

            // use rand::Rng;
            // let mut secret = [0_u8; 128];
            // rand::thread_rng().fill(&mut secret);
            // dbg!(hex::encode(secret));

            SessionLayer::new(store, decoded.as_slice()).with_secure(true)
        })
}

#[derive(Clone, Debug)]
struct BonsaiDBSessionStore {
    db_client: SharedDbClient,
}

#[async_trait]
impl SessionStore for BonsaiDBSessionStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> axum_sessions::async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let session = HangingStrategy::default()
            .execute(
                |db_client| async {
                    let id = id.clone();
                    async move {
                        let session: Option<Session> =
                            db_client.read().await.sessions().get_key(id).into().await?;

                        ApiResult::<_>::Ok(session)
                    }
                    .await
                },
                self.db_client.clone(),
            )
            .await??;

        Ok(session.and_then(Session::validate))
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> axum_sessions::async_session::Result<Option<String>> {
        let _: () = HangingStrategy::default()
            .execute(
                |db_client| async {
                    let session = session.clone();
                    async move {
                        db_client
                            .read()
                            .await
                            .sessions()
                            .set_key(session.id().to_string(), &session)
                            .await?;

                        ApiResult::<_>::Ok(())
                    }
                    .await
                },
                self.db_client.clone(),
            )
            .await??;

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> axum_sessions::async_session::Result {
        let _: () = HangingStrategy::default()
            .execute(
                |db_client| async {
                    let session = session.clone();
                    async move {
                        db_client
                            .read()
                            .await
                            .sessions()
                            .delete_key(session.id().to_string())
                            .await?;

                        ApiResult::<_>::Ok(())
                    }
                    .await
                },
                self.db_client.clone(),
            )
            .await??;

        Ok(())
    }

    async fn clear_store(&self) -> axum_sessions::async_session::Result {
        tracing::info!("clear session store");
        unimplemented!("find out how to clear session storage")
    }
}

#[derive(Clone)]
pub struct AppState {
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
    pub users_online: UsersOnline,
}

#[derive(Clone)]
pub struct UsersOnline {
    pub ips: Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, i32>>>,
    pub count_s: async_broadcast::Sender<i32>,
    pub count_r: async_broadcast::Receiver<i32>,
}

pub type SharedDbClient = Arc<RwLock<DbClient>>;
pub type SharedUsersOnline = Arc<RwLock<UsersOnline>>;

pub struct Application {
    port: u16,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = hyper::Result<()>> + Send>>,
    host: String,
    pub db_client: SharedDbClient,
}

impl Application {
    pub async fn build(conf: &Conf) -> Self {
        let sender_email = conf
            .env
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = conf.env.email_client.timeout();
        let email_client = Arc::new(EmailClient::new(
            conf.env.email_client.base_url.clone(),
            sender_email,
            conf.env.email_client.authorization_token.clone(),
            timeout,
        ));

        let address = format!("{}:{}", conf.env.host, conf.env.port);
        let listener = std::net::TcpListener::bind(&address).unwrap();
        tracing::info!("Listening on http://{}", address);
        let host = conf.env.host.clone();
        let port = listener.local_addr().unwrap().port();

        {
            let first_symbols_count = 5;
            let first_symbols_of_email_token =
                &(*conf.env.email_client.authorization_token.expose_secret())
                    [..first_symbols_count];
            tracing::info!(
                "First symbols of email token:{}",
                first_symbols_of_email_token
            );
        }

        pub fn run(
            conf: &Conf,
            listener: std::net::TcpListener,
            email_client: Arc<EmailClient>,
            base_url: String,
            db_client: SharedDbClient,
        ) -> impl std::future::Future<Output = hyper::Result<()>> {
            // simulate one value that many can await
            // dropping all unhandled values by ignoring Overflow error
            let (mut s, r) = async_broadcast::broadcast(1);
            s.set_overflow(true);

            let app_state = AppState {
                email_client,
                base_url,
                users_online: UsersOnline {
                    ips: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
                    count_s: s,
                    count_r: r,
                },
            };

            let app = router(conf, db_client).with_state(app_state);

            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service_with_connect_info::<UserConnectInfo>())
        }

        let db_client = SharedDbClient::new(RwLock::new(
            DbClient::configure(conf.db_client.clone())
                .await
                .expect("database must be available on deployment"),
        ));

        let server = Box::pin(run(
            conf,
            listener,
            email_client,
            conf.env.base_url.clone(),
            db_client.clone(),
        ));

        Self {
            server,
            port,
            host,
            db_client,
        }
    }

    // needs to consume to produce 1 server max, and because I don't know better
    pub fn server(self) -> impl std::future::Future<Output = hyper::Result<()>> + Send {
        self.server
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

#[derive(Clone, Debug)]
pub struct UserConnectInfo {
    pub remote_addr: std::net::SocketAddr,
}

impl axum::extract::connect_info::Connected<&hyper::server::conn::AddrStream> for UserConnectInfo {
    fn connect_info(target: &hyper::server::conn::AddrStream) -> Self {
        let remote_addr = target.remote_addr();
        Self { remote_addr }
    }
}
