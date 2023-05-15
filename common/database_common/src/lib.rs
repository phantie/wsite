use std::path::PathBuf;

pub const ADDR: &str = "0.0.0.0:3000";

pub fn storage_location() -> PathBuf {
    PathBuf::from("server-data.bonsaidb")
}

pub fn public_certificate_name() -> PathBuf {
    PathBuf::from("pinned-certificate.der")
}

pub mod schema;
