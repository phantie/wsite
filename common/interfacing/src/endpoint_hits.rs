use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EndpointHit {
    pub hashed_ip: String,
    pub endpoint: String,
    pub method: String,
    pub status: u16,
    pub timestamp: String,
}

impl EndpointHit {
    pub fn timestamp(&self) -> std::time::SystemTime {
        humantime::parse_rfc3339(&self.timestamp).unwrap()
    }

    pub fn formatted_now() -> String {
        let system_time = std::time::SystemTime::now();
        humantime::format_rfc3339(system_time).to_string()
    }

    pub fn hash_ip(ip: std::net::IpAddr) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let ip = ip.to_string();
        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FrontendEndpointHit {
    pub endpoint: String,
    pub status: u16,
}
