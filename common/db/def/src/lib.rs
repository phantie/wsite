pub const MANAGER_ADDR: &str = "localhost:3000";

pub fn storage_location() -> PathBuf {
    PathBuf::from("server-data.bonsaidb")
}

pub fn public_certificate_name() -> PathBuf {
    PathBuf::from("pinned-certificate.der")
}

use std::path::PathBuf;
