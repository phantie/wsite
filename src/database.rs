use bonsaidb::core::schema::Collection;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::AsyncDatabase;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use bonsaidb::core::connection::AsyncStorageConnection;
pub use bonsaidb::core::connection::StorageConnection;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;
pub use bonsaidb::local::config::Builder;
pub use bonsaidb::local::AsyncStorage;

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "subscriptions")]
pub struct Subscription {
    pub name: String,
    pub email: crate::domain::SubscriberEmail,
    pub status: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "users")]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

pub async fn storage(dir: &str, memory_only: bool) -> AsyncStorage {
    let mut configuration = StorageConfiguration::new(dir);
    configuration.memory_only = memory_only;
    let configuration = configuration.with_schema::<Subscription>().unwrap();
    let configuration = configuration.with_schema::<User>().unwrap();

    AsyncStorage::open(configuration).await.unwrap()
}

#[derive(Clone)]
pub struct Database {
    pub storage: Arc<AsyncStorage>,
    pub collections: Collections,
}

#[derive(Clone)]
pub struct Collections {
    pub subscriptions: AsyncDatabase,
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
                .create_database::<User>("users", true)
                .await
                .unwrap(),
        };

        Self {
            storage,
            collections,
        }
    }
}
