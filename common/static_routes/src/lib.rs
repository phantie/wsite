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
mod route_tests {
    use super::*;

    #[test]
    fn basic() {
        let routes = Routes::default();

        routes.api.health_check.get();

        assert_eq!(routes.api.health_check.get().postfix(), "/health_check");
    }
}
