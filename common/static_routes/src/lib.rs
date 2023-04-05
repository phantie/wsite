mod api_scope;
mod primitives;
mod root_scope;

pub use primitives::{Get, Post, RelativePath, Url};

#[allow(dead_code)]
#[derive(Default)]
pub struct Routes {
    pub api: api_scope::Routes,
    pub root: root_scope::Routes,
}

impl Routes {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn routes() -> Routes {
    Routes::new()
}

#[cfg(test)]
mod tests {
    #![allow(non_upper_case_globals)]
    use super::*;

    static localhost_dns: &str = "http://localhost";
    static localhost: &str = "http://127.0.0.1";
    static localhost_with_port: &str = "http://127.0.0.1:8000";
    static zeros_with_port: &str = "http://0.0.0.0:8000";
    static https: &str = "https://api-qwerty.digitalocean.com";

    static hosts: &[&'static str] = &[
        localhost_dns,
        localhost,
        localhost_with_port,
        zeros_with_port,
        https,
    ];

    #[test]
    fn test_health_check() {
        let route = routes().api.health_check.get();

        assert_eq!(route.postfix(), "/health_check");
        assert_eq!(route.prefix(), "/api");
        assert_eq!(route.complete(), "/api/health_check");
        for host in hosts {
            assert_eq!(
                route.complete_with_base(host),
                format!("{}/api/health_check", host)
            );
        }
    }

    #[test]
    fn test_home() {
        let route = routes().root.home.get();

        assert_eq!(route.postfix(), "/");
        assert_eq!(route.prefix(), "");
        assert_eq!(route.complete(), "/");
        for host in hosts {
            assert_eq!(route.complete_with_base(host), format!("{}/", host));
        }
    }
}
