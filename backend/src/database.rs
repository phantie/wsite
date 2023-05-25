pub use bonsaidb::core::connection::AsyncConnection;
pub use bonsaidb::core::connection::AsyncStorageConnection;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;
pub use common::db::schema;

use crate::configuration;
use crate::error::ApiResult;
use crate::timeout::TimeoutStrategy;
use bonsaidb::client::AsyncClient;
use bonsaidb::client::AsyncRemoteDatabase;
use hyper::StatusCode;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub async fn load_certificate(
    info_server: configuration::DbInfoServer,
) -> anyhow::Result<fabruic::Certificate> {
    let r = reqwest::get(info_server.cert_url).await?;

    match r.status() {
        StatusCode::OK => {
            let c: fabruic::Certificate = r.bytes().await?.to_vec().try_into()?;
            Ok(c)
        }
        StatusCode::NOT_FOUND => Err(anyhow::anyhow!("database has no volumes, or else"))?,
        status_code => Err(anyhow::anyhow!("unexpected status code {status_code}"))?,
    }

    // let f = std::fs::read("../database/http_server/server-data.bonsaidb/pinned-certificate.der")
    //     .unwrap()
    //     .try_into()
    //     .unwrap();
    // f
}

pub type SharedDbClient = Arc<tokio::sync::RwLock<DbClient>>;

// SCHEMA TWEAK
pub struct DbClient {
    conf: configuration::DbClientConf,
    liquid_state: DbClientLiquidState,
}

// Fields that change on reconfiguration moved here,
// because forgetting to also tweak fn reconfigure
// might result with buggy consequent reconfigurations
pub struct DbClientLiquidState {
    collections: DbCollections,
    sessions: AsyncRemoteDatabase,
    reconfiguration_id: u32,
    ping_handle: JoinHandle<()>,
    client: AsyncClient,
}

impl std::fmt::Debug for DbClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("DbClient [id {}]", self.reconfiguration_id()))
    }
}

#[derive(Clone)]
pub struct DbClientConf {
    pub quic_url: String,
    pub auth: DbClientAuth,
    pub certificate: fabruic::Certificate,
}

#[derive(Clone)]
pub enum DbClientAuth {
    Password(String),
    None,
}

pub struct RemoteClient {}

impl RemoteClient {
    pub async fn create(
        params: configuration::DbClientConf,
    ) -> anyhow::Result<bonsaidb::client::AsyncClient> {
        use bonsaidb::core::{
            admin::Role,
            connection::{Authentication, SensitiveString},
        };

        let params = match params {
            configuration::DbClientConf::Normal {
                password,
                quic_url,
                info_server,
            } => {
                let cert = load_certificate(info_server).await?;
                DbClientConf {
                    quic_url,
                    certificate: cert,
                    auth: DbClientAuth::Password(password),
                }
            }
            configuration::DbClientConf::Testing { quic_url, cert } => DbClientConf {
                quic_url,
                certificate: cert,
                auth: DbClientAuth::None,
            },
        };

        let client = bonsaidb::client::AsyncClient::build(
            bonsaidb::client::url::Url::parse(&params.quic_url).unwrap(),
        )
        .with_certificate(params.certificate)
        .build()?;

        let client = match params.auth {
            DbClientAuth::None => client,
            DbClientAuth::Password(password) => {
                let admin_password = password;

                let client = TimeoutStrategy::Once {
                    timeout: Duration::from_secs(5),
                }
                .execute(|| {
                    client.authenticate(Authentication::Password {
                        user: "admin".into(),
                        password: SensitiveString(admin_password.clone().into()),
                    })
                })
                .await??;

                let client = TimeoutStrategy::default()
                    .execute(|| Role::assume_identity_async("superuser", &client))
                    .await??;

                tracing::info!("Authenticated client as superuser");
                client
            }
        };

        Ok(client)
    }
}

#[derive(Clone)]
pub struct DbCollections {
    pub shapes: AsyncRemoteDatabase,
    pub users: AsyncRemoteDatabase,
    pub articles: AsyncRemoteDatabase,
    pub subs: AsyncRemoteDatabase,
}

impl DbCollections {
    pub async fn user_by_username(
        &self,
        username: impl AsRef<str>,
    ) -> ApiResult<Option<CollectionDocument<schema::User>>> {
        let mapped_users = self
            .users
            .view::<schema::UserByUsername>()
            .with_key(username.as_ref())
            .query_with_collection_docs()
            .await?;
        Ok(mapped_users.into_iter().next().map(|v| v.document.clone()))
    }

    pub async fn article_by_public_id(
        &self,
        public_id: impl AsRef<str>,
    ) -> ApiResult<Option<CollectionDocument<schema::Article>>> {
        let mapped = self
            .articles
            .view::<schema::ArticleByPublicID>()
            .with_key(public_id.as_ref())
            .query_with_collection_docs()
            .await?;
        Ok(mapped.into_iter().next().map(|v| v.document.clone()))
    }
}

pub type ClientResult<T> = std::result::Result<T, bonsaidb::core::Error>;

#[async_trait::async_trait]
pub trait Ping {
    async fn ping(&self) -> Result<(), anyhow::Error>;
}

#[async_trait::async_trait]
impl Ping for AsyncClient {
    async fn ping(&self) -> anyhow::Result<()> {
        match self.list_databases().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}

#[allow(non_upper_case_globals)]
static mut ReconfigurationID: AtomicU32 = AtomicU32::new(0);

impl DbClient {
    pub async fn configure(conf: configuration::DbClientConf) -> anyhow::Result<Self> {
        let client = RemoteClient::create(conf.clone()).await?;

        // try to solve a problem of client hanging forever
        // when not accessed for some time (empirically found more than 10 minutes)
        let ping_handle = {
            let client = client.clone();
            let ping_handle = tokio::task::spawn(async move {
                loop {
                    let ping = TimeoutStrategy::default().execute(|| client.ping()).await;

                    match ping {
                        Ok(o) => match o {
                            Ok(()) => tracing::info!("Ping database"),
                            Err(_e) => tracing::error!("Database error"),
                        },
                        Err(_) => tracing::error!("Database unreachable"),
                    }
                    // ping database every 5 minutes, to keep connection alive
                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
                }
            });
            ping_handle
        };

        // SCHEMA TWEAK
        let shapes = client.database::<schema::Shape>("shapes").await?;
        let users = client.database::<schema::User>("users").await?;
        let articles = client.database::<schema::Article>("articles").await?;
        let subs = client
            .database::<schema::Subscription>("subscriptions")
            .await?;
        let sessions = client.database::<()>("sessions").await?;

        let reconfiguration_id = unsafe { ReconfigurationID.load(Ordering::SeqCst) };
        unsafe { ReconfigurationID.fetch_add(1, Ordering::SeqCst) };

        Ok(Self {
            conf,
            liquid_state: DbClientLiquidState {
                sessions,
                collections: DbCollections {
                    shapes,
                    users,
                    articles,
                    subs,
                },
                client,
                reconfiguration_id,
                ping_handle,
            },
        })
    }

    pub async fn reconfigure(&mut self) -> anyhow::Result<()> {
        self.liquid_state.ping_handle.abort();
        let renewed = Self::configure(self.conf.clone()).await?;

        self.liquid_state = renewed.liquid_state;

        Ok(())
    }

    pub fn sessions(&self) -> AsyncRemoteDatabase {
        self.liquid_state.sessions.clone()
    }

    pub fn collections(&self) -> DbCollections {
        self.liquid_state.collections.clone()
    }

    pub fn reconfiguration_id(&self) -> u32 {
        self.liquid_state.reconfiguration_id
    }

    pub fn client(&self) -> AsyncClient {
        self.liquid_state.client.clone()
    }
}
