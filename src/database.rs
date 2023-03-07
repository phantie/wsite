use crate::configuration::get_configuration;

use bonsaidb::core::schema::Collection;
use bonsaidb::local::config::StorageConfiguration;
use serde::{Deserialize, Serialize};

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

pub fn storage_configuration(memory_only: bool) -> StorageConfiguration {
    let configuration = get_configuration();
    let mut conf = StorageConfiguration::new(configuration.database.dir);
    conf.memory_only = memory_only;
    conf.with_schema::<Subscription>().unwrap()
}

pub async fn storage_with_config(configuration: StorageConfiguration) -> AsyncStorage {
    AsyncStorage::open(configuration).await.unwrap()
}

pub async fn storage(memory_only: bool) -> AsyncStorage {
    storage_with_config(storage_configuration(memory_only)).await
}
