use crate::{configuration::Conf, database::*};
use static_routes::*;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_sessions::{
    async_session::{async_trait, Session, SessionStore},
    SessionLayer,
};
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

pub fn router(
    conf: &Conf,
    db_client: SharedDbClient,
    cozo_db: cozo::DbInstance,
) -> Router<AppState> {
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
        .route("/static/:path", get(serve_static));

    let ws_router = Router::new().route("/users_online", get(ws_users_online));

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
        .nest("/ws", ws_router)
        .fallback(fallback)
        .layer(CompressionLayer::new())
        .layer(AddExtensionLayer::new(db_client.clone()))
        .layer(AddExtensionLayer::new(cozo_db.clone()))
        .layer(request_tracing_layer)
        .layer({
            // let store = axum_sessions::async_session::MemoryStore::new();
            let store = BonsaiDBSessionStore {
                db: cozo_db.clone(),
            };

            let decoded = hex::decode(conf.env.session_secret.clone())
                .expect("Successful HEX Decoding of session secret");

            // use rand::Rng;
            // let mut secret = [0_u8; 128];
            // rand::thread_rng().fill(&mut secret);
            // dbg!(hex::encode(secret));

            SessionLayer::new(store, decoded.as_slice()).with_secure(true)
        })
}

#[derive(derivative::Derivative, Clone)]
#[derivative(Debug)]
struct BonsaiDBSessionStore {
    #[derivative(Debug = "ignore")]
    db: cozo::DbInstance,
}

use crate::cozo_db;

#[async_trait]
impl SessionStore for BonsaiDBSessionStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> axum_sessions::async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let session: Option<Session> = cozo_db::queries::find_session_by_id(&self.db, &id)?
            .map(|v| serde_json::from_str(&v).ok())
            .flatten()
            .and_then(Session::validate);

        Ok(session)
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> axum_sessions::async_session::Result<Option<String>> {
        cozo_db::queries::put_session(&self.db, session.id(), &serde_json::to_string(&session)?)?;
        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> axum_sessions::async_session::Result {
        cozo_db::queries::rm_session(&self.db, session.id())?;
        Ok(())
    }

    async fn clear_store(&self) -> axum_sessions::async_session::Result {
        tracing::info!("clear session store");
        unimplemented!("find out how to clear session storage")
    }
}

#[derive(Clone)]
pub struct AppState {
    pub users_online: UsersOnline,
}

pub type Cons = Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, i32>>>;

#[derive(Clone)]
pub struct UsersOnline {
    pub cons: Cons,
    pub con_count_s: async_broadcast::Sender<usize>,
    pub con_count_r: async_broadcast::Receiver<usize>,
}

impl UsersOnline {
    pub fn new() -> Self {
        // simulate one value that many can await
        // dropping all unhandled values by ignoring Overflow error
        let (mut con_count_s, con_count_r) = async_broadcast::broadcast(1);
        con_count_s.set_overflow(true);
        Self {
            cons: Cons::default(),
            con_count_s,
            con_count_r,
        }
    }

    pub async fn broadcast_con_count(&self, count: usize) {
        self.con_count_s
            .broadcast(count)
            .await
            .expect("Channels are always open");
    }
}

pub type SharedDbClient = Arc<RwLock<DbClient>>;

pub struct Application {
    port: u16,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = hyper::Result<()>> + Send>>,
    host: String,
    pub db_client: SharedDbClient,
}

impl Application {
    pub async fn build(conf: &Conf) -> Self {
        let address = format!("{}:{}", conf.env.host, conf.env.port);
        let listener = std::net::TcpListener::bind(&address).unwrap();
        tracing::info!("Listening on http://{}", address);
        let host = conf.env.host.clone();
        let port = listener.local_addr().unwrap().port();

        let db_client = Arc::new(RwLock::new(
            DbClient::configure(conf.db_client.clone())
                .await
                .expect("Database unavailable"),
        ));

        let cozo_db = crate::cozo_db::start_db();

        let app_state = AppState {
            users_online: UsersOnline::new(),
        };

        return Self {
            server: Box::pin(run(conf, listener, app_state, db_client.clone(), cozo_db)),
            port,
            host,
            db_client,
        };

        pub fn run(
            conf: &Conf,
            listener: std::net::TcpListener,
            app_state: AppState,
            db_client: SharedDbClient,
            cozo_db: cozo::DbInstance,
        ) -> impl std::future::Future<Output = hyper::Result<()>> {
            axum::Server::from_tcp(listener).unwrap().serve(
                router(conf, db_client, cozo_db)
                    .with_state(app_state)
                    .into_make_service_with_connect_info::<UserConnectInfo>(),
            )
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
        Self {
            remote_addr: target.remote_addr(),
        }
    }
}
