use std::sync::Arc;

use bonsaidb::core::schema::Collection;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::AsyncDatabase;
use serde::{Deserialize, Serialize};

pub use bonsaidb::core::connection::AsyncStorageConnection;
pub use bonsaidb::core::connection::StorageConnection;
pub use bonsaidb::core::document::CollectionDocument;
pub use bonsaidb::core::schema::SerializedCollection;
pub use bonsaidb::local::config::Builder;
pub use bonsaidb::local::AsyncStorage;

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "users")]
pub struct Subscription {
    pub name: String,
    pub email: String,
}

pub async fn storage(dir: &str, memory_only: bool) -> AsyncStorage {
    let mut configuration = StorageConfiguration::new(dir);
    configuration.memory_only = memory_only;
    let configuration = configuration.with_schema::<Subscription>().unwrap();

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
}

impl Database {
    pub async fn init(storage: Arc<AsyncStorage>) -> Self {
        let collections = Collections {
            subscriptions: storage
                .create_database::<Subscription>("users", true)
                .await
                .unwrap(),
        };

        Self {
            storage,
            collections,
        }
    }
}
