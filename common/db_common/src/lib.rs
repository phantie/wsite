use std::path::PathBuf;

pub const MANAGER_ADDR: &str = "localhost:3000";

pub fn storage_location() -> PathBuf {
    PathBuf::from("server-data.bonsaidb")
}

pub fn public_certificate_name() -> PathBuf {
    PathBuf::from("pinned-certificate.der")
}

pub mod init;
pub mod schema;
