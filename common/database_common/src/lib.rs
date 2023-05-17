use std::path::PathBuf;

pub const ADDR: &str = "localhost:3000";

pub fn storage_location() -> PathBuf {
    PathBuf::from("server-data.bonsaidb")
}

pub fn public_certificate_name() -> PathBuf {
    PathBuf::from("pinned-certificate.der")
}

mod pointer;
pub mod schema;
use pointer::DatabasePointer;

pub fn users_pointer() -> DatabasePointer<schema::User> {
    DatabasePointer::new("users".into())
}
