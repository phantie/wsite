use crate::configuration::{get_env, Conf};
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

pub fn router(conf: &Conf, db: cozo::DbInstance) -> Router {
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
        .route("/static/*path", get(serve_static))
        // .route("/admin/endpoint_hits", get(endpoint_hits))
        .route(
            routes.admin.endpoint_hits.get().postfix(),
            get(endpoint_hits),
        )
        .route(
            routes.admin.endpoint_hits.grouped.get().postfix(),
            get(endpoint_hits_grouped),
        )
        // .route("/endpoint_hits/frontend", post(frontend_endpoint_hit))
        .route(
            routes.endpoint_hits.frontend.post().postfix(),
            post(frontend_endpoint_hit),
        )
        .route(
            routes.endpoint_hits.github.profile.get().postfix(),
            get(github_hit),
        )
        .route(
            routes.endpoint_hits.github.wsite.get().postfix(),
            get(wsite_github_hit),
        )
        // TODO investigate why POST on /lobby gives 200
        .route("/snake/lobby", post(snake::create_lobby))
        .route("/snake/lobby/:name", get(snake::get_lobby))
        .route("/snake/ws", get(snake::ws::ws));

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
        .layer(axum::middleware::from_fn(endpoint_hit_middleware))
        .layer(AddExtensionLayer::new(db.clone()))
        .layer(AddExtensionLayer::new(UsersOnline::new()))
        .layer(AddExtensionLayer::new(mp_snake::Lobbies::default()))
        .layer(request_tracing_layer)
        .layer({
            // let store = axum_sessions::async_session::MemoryStore::new();
            let store = BonsaiDBSessionStore { db: db.clone() };

            let decoded = hex::decode(conf.env.session_secret.clone())
                .expect("Successful HEX Decoding of session secret");

            // use rand::Rng;
            // let mut secret = [0_u8; 128];
            // rand::thread_rng().fill(&mut secret);
            // dbg!(hex::encode(secret));

            SessionLayer::new(store, decoded.as_slice()).with_secure(true)
        })
}

// TODO move out
pub mod mp_snake {
    use interfacing::snake::{LobbyName, UserName};

    #[derive(Clone)]
    pub struct Player {
        #[allow(unused)]
        name: UserName,
    }

    #[derive(Clone)]
    pub struct Lobby {
        #[allow(unused)]
        pub name: LobbyName,
        #[allow(unused)]
        players: Vec<Player>,
    }

    impl Lobby {
        pub fn new(name: LobbyName) -> Self {
            Self {
                name,
                players: vec![],
            }
        }
    }

    // TODO Value In HashMap (Lobby) Should be wrapped in RwLock or Something
    // to increase concurrency by accessing through outer RwLock using read
    type Inner = std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<LobbyName, Lobby>>>;

    #[derive(Clone, Default)]
    pub struct Lobbies(Inner);

    impl std::ops::Deref for Lobbies {
        type Target = Inner;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Lobbies {
        pub async fn get(&self, name: &LobbyName) -> Option<Lobby> {
            self.read().await.get(name).cloned()
        }

        pub async fn insert_if_missing(&self, lobby: Lobby) -> Result<(), String> {
            use std::collections::hash_map::Entry;
            let mut w_lock = self.write().await;

            match w_lock.entry(lobby.name.clone()) {
                Entry::Occupied(_) => Err("Lobby with this name already exists".into()),
                Entry::Vacant(_) => {
                    w_lock.insert(lobby.name.clone(), lobby);
                    Ok(())
                }
            }
        }

        #[allow(dead_code)]
        async fn insert(&self, lobby: Lobby) {
            self.write().await.insert(lobby.name.clone(), lobby);
        }
    }

    pub mod ws {
        use interfacing::snake::UserName;

        pub type Cons<S> =
            std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, S>>>;

        #[derive(Clone, Default)]
        pub struct State {
            #[allow(unused)]
            pub user_name: Option<UserName>,
        }
    }
}

async fn endpoint_hit_middleware<B>(
    h: hyper::HeaderMap,
    axum::extract::Extension(db): axum::extract::Extension<cozo::DbInstance>,
    // TODO request hangs if this extractor is used
    // session: axum_sessions::extractors::ReadableSession,
    request: hyper::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    let endpoint = request.uri().to_string();
    let method = request.method().to_string();

    let response = next.run(request).await;
    let routes = routes().api;

    let skip_endpoint_starts = [
        "/favicon.ico".to_string(),
        "/api/admin/session".into(),
        "/_trunk/ws".into(),
        // after upgrade ip defaults to localhost
        // now not worth to bother implementing correctly
        "/ws/users_online".into(),
        routes.admin.endpoint_hits.get().complete().into(),
        routes.admin.endpoint_hits.grouped.get().complete().into(),
        routes.endpoint_hits.frontend.post().complete().into(),
        routes.endpoint_hits.github.profile.get().complete().into(),
        routes.endpoint_hits.github.wsite.get().complete().into(),
    ];

    let js_file = endpoint.starts_with("/frontend") && endpoint.ends_with(".js");
    let wasm_file = endpoint.starts_with("/frontend") && endpoint.ends_with(".wasm");
    let favicon = endpoint.starts_with("/favicon") && endpoint.ends_with(".ico");

    // skip logged in hits in prod
    // let skip = crate::authentication::reject_anonymous_users(&session).is_ok() && get_env().prod();
    let skip = skip_endpoint_starts
        .into_iter()
        .any(|start| endpoint.starts_with(&start))
        || js_file
        || wasm_file
        || favicon;

    if !skip {
        let system_time = interfacing::EndpointHit::formatted_now();

        let ip = ip_address(&h);
        let hashed_ip = if get_env().local() {
            ip.to_string()
        } else {
            interfacing::EndpointHit::hash_ip(ip)
        };

        let status = response.status().as_u16();

        let _result = db::q::put_endpoint_hit(
            &db,
            interfacing::EndpointHit {
                hashed_ip,
                endpoint,
                method,
                status,
                timestamp: system_time,
            },
        )
        .map_err(|e| tracing::error!("{e:?}"));
    }

    response
}

#[derive(derivative::Derivative, Clone)]
#[derivative(Debug)]
struct BonsaiDBSessionStore {
    #[derivative(Debug = "ignore")]
    db: cozo::DbInstance,
}

use crate::db;

#[async_trait]
impl SessionStore for BonsaiDBSessionStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> axum_sessions::async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let session: Option<Session> = db::q::find_session_by_id(&self.db, &id)?
            .map(|v| serde_json::from_str(&v).ok())
            .flatten()
            .and_then(Session::validate);

        Ok(session)
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> axum_sessions::async_session::Result<Option<String>> {
        db::q::put_session(&self.db, session.id(), &serde_json::to_string(&session)?)?;
        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> axum_sessions::async_session::Result {
        db::q::rm_session(&self.db, session.id())?;
        Ok(())
    }

    async fn clear_store(&self) -> axum_sessions::async_session::Result {
        tracing::info!("clear session store");
        unimplemented!("find out how to clear session storage")
    }
}

pub type Cons<S> = Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, S>>>;

#[derive(Clone)]
pub struct UsersOnline {
    pub cons: Cons<i32>,
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

pub struct Application {
    port: u16,
    server: std::pin::Pin<Box<dyn std::future::Future<Output = hyper::Result<()>> + Send>>,
    host: String,
    db: cozo::DbInstance,
}

impl Application {
    pub async fn build(conf: &Conf) -> Self {
        let address = format!("{}:{}", conf.env.host, conf.env.port);
        let listener = std::net::TcpListener::bind(&address).unwrap();
        tracing::info!("Listening on http://{}", address);
        let host = conf.env.host.clone();
        let port = listener.local_addr().unwrap().port();

        // let db = &DbInstance::new("sqlite", "testing.db", Default::default()).unwrap();
        let db = conf.env.db.db_instance();
        let db = crate::db::start_db(db);

        return Self {
            server: Box::pin(run(conf, listener, db.clone())),
            port,
            host,
            db,
        };

        pub fn run(
            conf: &Conf,
            listener: std::net::TcpListener,
            db: cozo::DbInstance,
        ) -> impl std::future::Future<Output = hyper::Result<()>> {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(router(conf, db).into_make_service_with_connect_info::<UserConnectInfo>())
        }
    }

    // needs to consume to produce 1 server max, and because I don't know better
    pub fn server(self) -> impl std::future::Future<Output = hyper::Result<()>> + Send {
        self.server
    }

    pub fn db(&self) -> cozo::DbInstance {
        self.db.clone()
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
    remote_addr: std::net::SocketAddr,
}

impl UserConnectInfo {
    pub fn socket_addr(&self, headers: &hyper::HeaderMap) -> std::net::SocketAddr {
        let ip = ip_address(headers);
        let mut sock = self.remote_addr;
        // rewrite ip address because server may be behind reverse proxy
        sock.set_ip(ip);
        sock
    }
}

impl axum::extract::connect_info::Connected<&hyper::server::conn::AddrStream> for UserConnectInfo {
    fn connect_info(target: &hyper::server::conn::AddrStream) -> Self {
        Self {
            remote_addr: target.remote_addr(),
        }
    }
}

fn get_origin(h: &hyper::HeaderMap) -> Option<std::net::IpAddr> {
    h.get("origin")
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| url::Url::parse(v).ok())
        .flatten()
        .map(|v| v.host_str().map(|v| v.to_owned()))
        .flatten()
        .map(|v| v.parse::<std::net::IpAddr>().ok())
        .flatten()
}

fn get_referer(h: &hyper::HeaderMap) -> Option<std::net::IpAddr> {
    h.get("referer")
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| url::Url::parse(v).ok())
        .flatten()
        .map(|v| v.host_str().map(|v| v.to_owned()))
        .flatten()
        .map(|v| v.parse::<std::net::IpAddr>().ok())
        .flatten()
}

fn get_x_forwarded_for(h: &hyper::HeaderMap) -> Option<std::net::IpAddr> {
    h.get("x-forwarded-for")
        .map(|v| v.to_str().ok())
        .flatten()
        .map(|v| v.split(",").map(|v| v.trim()).last())
        .flatten()
        .map(|v| v.parse::<std::net::IpAddr>().ok())
        .flatten()
}

// TODO refactor into extractor
pub fn ip_address(h: &hyper::HeaderMap) -> std::net::IpAddr {
    get_x_forwarded_for(h) // when behind reverse proxy
        .or_else(|| get_referer(h)) // when local not ws
        .or_else(|| get_origin(h)) // when local ws
        // fallback if buggy code above
        .unwrap_or_else(|| {
            tracing::error!("should have gotten IP by here");
            "127.0.0.1".parse().unwrap()
        })
}
