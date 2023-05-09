pub use bonsaidb::core::connection::AsyncConnection;
pub use bonsaidb::core::connection::AsyncStorageConnection;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;
pub use bonsaidb::local::config::Builder;
pub use bonsaidb::local::AsyncStorage;

use crate::timeout::TimeoutStrategy;
use bonsaidb::client::Client;
use bonsaidb::core::document::BorrowedDocument;
use bonsaidb::core::document::Emit;
use bonsaidb::core::schema::Collection;
use bonsaidb::core::schema::ReduceResult;
use bonsaidb::core::schema::View;
use bonsaidb::core::schema::ViewMapResult;
use bonsaidb::core::schema::ViewMappedValue;
use bonsaidb::core::schema::ViewSchema;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::AsyncDatabase;
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

    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.status, 1)
    }

    fn version(&self) -> u64 {
        3
    }
}

#[derive(Debug, Clone, View)]
#[view(collection = Subscription, key = String, value = u32, name = "by-token")]
pub struct SubscriptionByToken;

impl ViewSchema for SubscriptionByToken {
    type View = Self;

    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.token, 1)
    }

    fn version(&self) -> u64 {
        2
    }

    fn unique(&self) -> bool {
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "users",  views = [UserByUsername])]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, View)]
#[view(collection = User, key = String, value = u32, name = "by-username")]
pub struct UserByUsername;

impl ViewSchema for UserByUsername {
    type View = Self;

    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let user = User::document_contents(document)?;
        document.header.emit_key_and_value(user.username, 1)
    }

    fn version(&self) -> u64 {
        2
    }

    fn unique(&self) -> bool {
        true
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<Self::View>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        Ok(mappings.iter().map(|mapping| mapping.value).sum())
    }
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "articles", views = [ArticleByPublicID])]
pub struct Article {
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

#[derive(Debug, Clone, View)]
#[view(collection = Article, key = String, value = u32, name = "by-public-id")]
pub struct ArticleByPublicID;

impl ViewSchema for ArticleByPublicID {
    type View = Self;

    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let user = Article::document_contents(document)?;
        document.header.emit_key_and_value(user.public_id, 1)
    }

    fn unique(&self) -> bool {
        true
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<Self::View>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        Ok(mappings.iter().map(|mapping| mapping.value).sum())
    }
}

pub async fn storage(dir: &str, memory_only: bool) -> AsyncStorage {
    let mut configuration = StorageConfiguration::new(dir);
    configuration.memory_only = memory_only;
    let configuration = configuration.with_schema::<Subscription>().unwrap();
    let configuration = configuration.with_schema::<User>().unwrap();
    let configuration = configuration.with_schema::<Article>().unwrap();
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
    pub users: AsyncDatabase,
    pub articles: AsyncDatabase,
}

impl Database {
    pub async fn init(storage: Arc<AsyncStorage>) -> Self {
        let collections = Collections {
            subscriptions: storage
                .create_database::<Subscription>("subscriptions", true)
                .await
                .unwrap(),
            users: storage
                .create_database::<User>("users", true)
                .await
                .unwrap(),
            articles: storage
                .create_database::<Article>("articles", true)
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

pub fn load_certificate() -> fabruic::Certificate {
    // include_bytes!("/Users/phantie/Desktop/ahh/pinned-certificate.der")
    include_bytes!("../../database/http_server/server-data.bonsaidb/pinned-certificate.der")
        .to_vec()
        .try_into()
        .unwrap()
}

use database_common::schema;

pub struct RemoteDatabase {
    client: bonsaidb::client::Client,
    name: String,
    client_params: RemoteClientParams,
    pub collections: RemoteCollections,
    pub id: u32,
    ping_handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct RemoteClientParams {
    pub url: String,
    pub password: String,
}

pub struct RemoteClient {}

impl RemoteClient {
    pub async fn create(params: RemoteClientParams) -> anyhow::Result<bonsaidb::client::Client> {
        use bonsaidb::core::{
            admin::Role,
            connection::{Authentication, SensitiveString},
        };

        let client = bonsaidb::client::Client::build(
            bonsaidb::client::url::Url::parse(&params.url).unwrap(),
        )
        .with_certificate(load_certificate())
        .finish()?;

        let admin_password = params.password;

        let client = TimeoutStrategy::Once {
            timeout: Duration::from_secs(5),
        }
        .execute(|| {
            client.authenticate(
                "admin",
                Authentication::Password(SensitiveString(admin_password.clone().into())),
            )
        })
        .await??;

        let client = TimeoutStrategy::default()
            .execute(|| Role::assume_identity_async("superuser", &client))
            .await??;

        tracing::info!("Authenticated client as superuser");

        Ok(client)
    }
}

#[derive(Clone)]
pub struct RemoteCollections {
    pub shapes: bonsaidb::client::RemoteDatabase,
}

impl AsRef<bonsaidb::client::Client> for RemoteDatabase {
    fn as_ref(&self) -> &bonsaidb::client::Client {
        &self.client
    }
}

pub type ClientResult<T> = std::result::Result<T, bonsaidb::core::Error>;

#[async_trait::async_trait]
pub trait Ping {
    async fn ping(&self) -> Result<(), anyhow::Error>;
}

#[async_trait::async_trait]
impl Ping for Client {
    async fn ping(&self) -> anyhow::Result<()> {
        match self.list_databases().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)?,
        }
    }
}

#[allow(non_upper_case_globals)]
static mut RemoteDatabaseID: AtomicU32 = AtomicU32::new(0);

impl RemoteDatabase {
    pub async fn configure(name: &str, params: RemoteClientParams) -> anyhow::Result<Self> {
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
                    // ping database every 5 minutes, to have connection alive
                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
                }
            });
            ping_handle
        };

        let shapes = client.create_database::<schema::Shape>(name, true).await?;

        let id = unsafe { RemoteDatabaseID.load(Ordering::SeqCst) };
        unsafe { RemoteDatabaseID.fetch_add(1, Ordering::SeqCst) };

        Ok(Self {
            client,
            name: name.into(),
            collections: RemoteCollections { shapes },
            client_params: params,
            ping_handle,
            id,
        })
    }

    pub async fn reconfigure(&mut self) -> anyhow::Result<()> {
        self.ping_handle.abort();
        let renewed = Self::configure(&self.name, self.client_params.clone()).await?;

        self.client = renewed.client;
        self.collections = renewed.collections;
        self.id = renewed.id;
        self.ping_handle = renewed.ping_handle;

        Ok(())
    }

    pub async fn request_database<DB: bonsaidb::core::schema::Schema>(
        &self,
    ) -> ClientResult<bonsaidb::client::RemoteDatabase> {
        self.client.database::<DB>(&self.name).await
    }

    pub async fn request_create_collection<DB: bonsaidb::core::schema::Schema>(
        &self,
        only_if_needed: bool,
    ) -> ClientResult<bonsaidb::client::RemoteDatabase> {
        self.client
            .create_database::<DB>(&self.name, only_if_needed)
            .await
    }
}
