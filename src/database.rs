use crate::configuration::get_configuration;

use bonsaidb::core::schema::Collection;
use bonsaidb::local::config::StorageConfiguration;
use bonsaidb::local::Storage;
use serde::{Deserialize, Serialize};

pub use bonsaidb::core::connection::StorageConnection;
pub use bonsaidb::core::schema::SerializedCollection;
pub use bonsaidb::local::config::Builder;

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

pub fn storage_with_config(configuration: StorageConfiguration) -> Storage {
    Storage::open(configuration).expect("Should succeed")
}

pub fn storage(memory_only: bool) -> Storage {
    storage_with_config(storage_configuration(memory_only))
}
