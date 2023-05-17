pub use bonsaidb::core::connection::AsyncConnection;
pub use bonsaidb::core::connection::AsyncStorageConnection;
use bonsaidb::core::connection::HasSession;
use bonsaidb::core::connection::SensitiveString;
use bonsaidb::core::connection::SessionAuthentication;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;
pub use bonsaidb::local::config::Builder;
pub use bonsaidb::local::AsyncStorage;
use tokio::sync::RwLock;

use crate::configuration::get_configuration;
use crate::timeout::TimeoutStrategy;
use bonsaidb::client::AsyncClient;
use bonsaidb::core::document::BorrowedDocument;
use bonsaidb::core::document::Emit;
use bonsaidb::core::schema::view::ViewUpdatePolicy;
use bonsaidb::core::schema::Collection;
use bonsaidb::core::schema::MapReduce;
use bonsaidb::core::schema::View;
use bonsaidb::core::schema::ViewMapResult;
use bonsaidb::core::schema::ViewSchema;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::AsyncDatabase;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::unreachable;
use tokio::task::JoinHandle;

use database_common::schema;

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "subscriptions", views = [SubscriptionByStatus, SubscriptionByToken])]
pub struct Subscription {
    pub name: String,
    pub email: crate::domain::SubscriberEmail,
    pub status: String,
    pub token: String,
}

#[derive(Debug, Clone, View)]
#[view(collection = Subscription, key = String, value = u32, name = "by-status")]
pub struct SubscriptionByStatus;

impl ViewSchema for SubscriptionByStatus {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;
}

impl MapReduce for SubscriptionByStatus {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.status, 1)
    }
}

#[derive(Debug, Clone, View)]
#[view(collection = Subscription, key = String, value = u32, name = "by-token")]
pub struct SubscriptionByToken;

impl ViewSchema for SubscriptionByToken {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;

    fn update_policy(&self) -> ViewUpdatePolicy {
        ViewUpdatePolicy::Unique
    }
}

impl MapReduce for SubscriptionByToken {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.token, 1)
    }
}

pub async fn storage(dir: &str, memory_only: bool) -> AsyncStorage {
    let mut configuration = StorageConfiguration::new(dir);
    configuration.memory_only = memory_only;
    let configuration = configuration.with_schema::<Subscription>().unwrap();
    let configuration = configuration.with_schema::<schema::User>().unwrap();
    let configuration = configuration.with_schema::<()>().unwrap();

    AsyncStorage::open(configuration).await.unwrap()
}

#[derive(Clone, Debug)]
pub struct Database {
    pub storage: Arc<AsyncStorage>,
    pub collections: Collections,
    pub sessions: AsyncDatabase,
}

#[derive(Clone, Debug)]
pub struct Collections {
    pub subscriptions: AsyncDatabase,
    // TODO can't remove because god-forgotten tests that access this field
    pub users: AsyncDatabase,
}

impl Database {
    pub async fn init(storage: Arc<AsyncStorage>) -> Self {
        let collections = Collections {
            subscriptions: storage
                .create_database::<Subscription>("subscriptions", true)
                .await
                .unwrap(),
            users: storage
                .create_database::<schema::User>("users", true)
                .await
                .unwrap(),
        };

        let sessions = storage
            .create_database::<()>("sessions", true)
            .await
            .unwrap();

        Self {
            storage,
            collections,
            sessions,
        }
    }
}

pub async fn load_certificate() -> fabruic::Certificate {
    let conf = get_configuration();

    // TODO if database does not exists, it panics
    let r = reqwest::get(format!("http://{}:4000/cert", conf.database.host))
        .await
        .unwrap();

    match r.status() {
        StatusCode::OK => {
            let c: fabruic::Certificate = r.bytes().await.unwrap().to_vec().try_into().unwrap();
            c
        }
        StatusCode::NOT_FOUND => panic!("database has no volumes, or else"),
        _ => unimplemented!(),
    }

    // let f = std::fs::read("../database/http_server/server-data.bonsaidb/pinned-certificate.der")
    //     .unwrap()
    //     .try_into()
    //     .unwrap();
    // f
}

pub struct RemoteDatabase {
    pub bare_client: RwLock<RemoteClient>,
    name: String,
    client_params: RemoteClientParams,
    pub id: u32,
    ping_handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct RemoteClientParams {
    pub url: String,
    pub password: SensitiveString,
}

#[derive(Clone)]
pub struct RemoteClient {
    inner: bonsaidb::client::AsyncClient,
    password: SensitiveString,
}

impl RemoteClient {
    pub async fn create(params: RemoteClientParams) -> anyhow::Result<Self> {
        let client = bonsaidb::client::AsyncClient::build(
            bonsaidb::client::url::Url::parse(&params.url).unwrap(),
        )
        .with_certificate(load_certificate().await)
        .with_connect_timeout(Duration::from_secs(5))
        .with_request_timeout(Duration::from_secs(3))
        .build()?;

        Ok(Self {
            inner: client,
            password: params.password,
        })
    }
}

// impl AsRef<AsyncClient> for RemoteDatabase {
//     fn as_ref(&self) -> &AsyncClient {
//         &self.bare_client.read().await.inner
//     }
// }

pub type ClientResult<T> = std::result::Result<T, bonsaidb::core::Error>;

#[async_trait::async_trait]
pub trait Ping {
    async fn ping(&self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl Ping for AsyncClient {
    async fn ping(&self) -> anyhow::Result<()> {
        // match self.list_databases().await {
        //     Ok(_) => Ok(()),
        //     Err(e) => Err(e)?,
        // }
        Ok(())
    }
}

#[allow(non_upper_case_globals)]
static mut RemoteDatabaseID: AtomicU32 = AtomicU32::new(0);

impl RemoteDatabase {
    pub async fn shapes(&self) -> anyhow::Result<bonsaidb::client::AsyncRemoteDatabase> {
        Ok(self
            .client()
            .await?
            .database::<schema::Shape>("shapes")
            .await?)
    }

    pub async fn users(&self) -> anyhow::Result<bonsaidb::client::AsyncRemoteDatabase> {
        Ok(self
            .client()
            .await?
            .database::<schema::User>("users")
            .await?)
    }

    pub async fn articles(&self) -> anyhow::Result<bonsaidb::client::AsyncRemoteDatabase> {
        Ok(self
            .client()
            .await?
            .database::<schema::Article>("articles")
            .await?)
    }

    pub async fn client(&self) -> anyhow::Result<bonsaidb::client::AsyncClient> {
        use bonsaidb::core::{admin::Role, connection::Authentication};

        let client = self.bare_client.read().await.inner.clone();

        // if let Some(_session) = client.session() {
        //     dbg!(_session);
        //     return Ok(client);
        // }

        match client.session() {
            None => {}
            Some(bonsaidb::core::connection::Session { authentication, .. }) => {
                match authentication {
                    SessionAuthentication::None => {}
                    _ => return Ok(client),
                }
            }
        }

        let admin_password = self.bare_client.read().await.password.clone();

        let client = TimeoutStrategy::Once {
            timeout: Duration::from_secs(5),
        }
        .execute(|| {
            client.authenticate(Authentication::Password {
                user: "admin".into(),
                password: admin_password.clone(),
            })
        })
        .await??;

        let client = TimeoutStrategy::default()
            .execute(|| Role::assume_identity_async("superuser", &client))
            .await??;

        self.bare_client.write().await.inner = client.clone();

        tracing::info!("Authenticated client as superuser");
        Ok(client)
    }

    pub async fn configure(name: &str, params: RemoteClientParams) -> anyhow::Result<Self> {
        let client = RemoteClient::create(params.clone()).await?;

        // try to solve a problem of client hanging forever
        // when not accessed for some time (empirically found more than 10 minutes)
        let ping_handle = {
            let client = client.clone();
            let ping_handle = tokio::task::spawn(async move {
                loop {
                    let ping = TimeoutStrategy::default()
                        .execute(|| client.inner.ping())
                        .await;

                    match ping {
                        Ok(o) => match o {
                            Ok(()) => tracing::info!("Ping database"),
                            Err(_e) => tracing::error!("Database error"),
                        },
                        Err(_) => tracing::error!("Database unreachable"),
                    }
                    // ping database every 5 minutes, to have connection alive
                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
                }
            });
            ping_handle
        };

        // SCHEMA TWEAK

        let id = unsafe { RemoteDatabaseID.load(Ordering::SeqCst) };
        unsafe { RemoteDatabaseID.fetch_add(1, Ordering::SeqCst) };

        Ok(Self {
            bare_client: RwLock::new(client),
            name: name.into(),
            client_params: params,
            ping_handle,
            id,
        })
    }

    pub async fn reconfigure(&mut self) -> anyhow::Result<()> {
        self.ping_handle.abort();
        let renewed = Self::configure(&self.name, self.client_params.clone()).await?;

        self.bare_client = renewed.bare_client;
        self.id = renewed.id;
        self.ping_handle = renewed.ping_handle;

        Ok(())
    }
}
