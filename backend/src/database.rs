pub use bonsaidb::core::connection::AsyncConnection;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;

use crate::configuration::get_configuration;
use crate::timeout::TimeoutStrategy;
use bonsaidb::client::AsyncClient;
use bonsaidb::core::connection::AsyncStorageConnection;
use bonsaidb::core::document::BorrowedDocument;
use bonsaidb::core::document::Emit;
use bonsaidb::core::schema::view::ViewUpdatePolicy;
use bonsaidb::core::schema::Collection;
use bonsaidb::core::schema::MapReduce;
use bonsaidb::core::schema::View;
use bonsaidb::core::schema::ViewMapResult;
use bonsaidb::core::schema::ViewSchema;
use bonsaidb::local::config::Builder;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::AsyncDatabase;
use bonsaidb::local::AsyncStorage;
use database_common::schema;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

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

    AsyncStorage::open(configuration).await.unwrap()
}

#[derive(Clone, Debug)]
pub struct Database {
    pub storage: Arc<AsyncStorage>,
    pub collections: Collections,
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

        Self {
            storage,
            collections,
        }
    }
}

pub async fn load_certificate() -> anyhow::Result<fabruic::Certificate> {
    let conf = get_configuration();

    let r = reqwest::get(format!("http://{}:4000/cert", conf.database.host)).await?;

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

// SCHEMA TWEAK
pub struct RemoteDatabase {
    pub sessions: bonsaidb::client::AsyncRemoteDatabase,
    pub collections: RemoteCollections,

    client: AsyncClient,
    client_params: RemoteClientParams,
    reconfiguration_id: u32,
    ping_handle: JoinHandle<()>,
}

impl std::fmt::Debug for RemoteDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DB [Reconfiguration ID {}]",
            self.reconfiguration_id()
        ))
    }
}

#[derive(Clone)]
pub struct RemoteClientParams {
    pub url: String,
    pub password: String,
}

pub struct RemoteClient {}

impl RemoteClient {
    pub async fn create(
        params: RemoteClientParams,
    ) -> anyhow::Result<bonsaidb::client::AsyncClient> {
        use bonsaidb::core::{
            admin::Role,
            connection::{Authentication, SensitiveString},
        };

        let client = bonsaidb::client::AsyncClient::build(
            bonsaidb::client::url::Url::parse(&params.url).unwrap(),
        )
        .with_certificate(load_certificate().await?)
        .build()?;

        let admin_password = params.password;

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

        Ok(client)
    }
}

pub struct RemoteCollections {
    pub shapes: bonsaidb::client::AsyncRemoteDatabase,
    pub users: bonsaidb::client::AsyncRemoteDatabase,
    pub articles: bonsaidb::client::AsyncRemoteDatabase,
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

impl RemoteDatabase {
    pub async fn configure(params: RemoteClientParams) -> anyhow::Result<Self> {
        let client = RemoteClient::create(params.clone()).await?;

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
        let sessions = client.database::<()>("sessions").await?;

        let reconfiguration_id = unsafe { ReconfigurationID.load(Ordering::SeqCst) };
        unsafe { ReconfigurationID.fetch_add(1, Ordering::SeqCst) };

        Ok(Self {
            client,
            collections: RemoteCollections {
                shapes,
                users,
                articles,
            },
            sessions,
            client_params: params,
            ping_handle,
            reconfiguration_id,
        })
    }

    pub async fn reconfigure(&mut self) -> anyhow::Result<()> {
        self.ping_handle.abort();
        let renewed = Self::configure(self.client_params.clone()).await?;

        self.client = renewed.client;
        self.collections = renewed.collections;
        self.reconfiguration_id = renewed.reconfiguration_id;
        self.ping_handle = renewed.ping_handle;
        self.sessions = renewed.sessions;

        Ok(())
    }

    pub fn reconfiguration_id(&self) -> u32 {
        self.reconfiguration_id
    }
}
