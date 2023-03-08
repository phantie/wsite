use bonsaidb::core::schema::Collection;
use bonsaidb::local::config::StorageConfiguration;
use serde::{Deserialize, Serialize};

pub use bonsaidb::core::connection::AsyncStorageConnection;
pub use bonsaidb::core::connection::StorageConnection;
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
