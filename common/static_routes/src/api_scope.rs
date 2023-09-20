#[allow(unused_imports)]
use crate::primitives::{Get, Post, Url};
use macros::*;

#[derive(Default)]
pub struct Routes {
    pub health_check: HealthCheck,
    pub login: Login,
    pub admin: Admin,
    pub articles: Articles,
    pub endpoint_hits: EndpointHits,
}

#[derive(Default)]
pub struct EndpointHits {
    pub frontend: EndpointHitsFrontend,
    pub github: EndpointHitsGithub,
}

#[derive(Default, Post)]
pub struct EndpointHitsFrontend;

impl Url for EndpointHitsFrontend {
    fn postfix(&self) -> &str {
        "/endpoint_hits/frontend"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default)]
pub struct EndpointHitsGithub {
    pub profile: EndpointHitsGithubProfile,
    pub wsite: EndpointHitsGithubWsite,
}

#[derive(Default, Get)]
pub struct EndpointHitsGithubProfile;

impl Url for EndpointHitsGithubProfile {
    fn postfix(&self) -> &str {
        "/endpoint_hits/github"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct EndpointHitsGithubWsite;

impl Url for EndpointHitsGithubWsite {
    fn postfix(&self) -> &str {
        "/endpoint_hits/github/wsite"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default)]
pub struct Admin {
    pub password: AdminPassword,
    pub logout: AdminLogout,
    pub session: AdminSession,
    pub articles: AdminArticles,
    pub endpoint_hits: AdminEndpointHits,
}

#[derive(Default, Get)]
pub struct HealthCheck;

impl Url for HealthCheck {
    fn postfix(&self) -> &str {
        "/health_check"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct Login;

impl Url for Login {
    fn postfix(&self) -> &str {
        "/login"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct AdminPassword;

impl Url for AdminPassword {
    fn postfix(&self) -> &str {
        "/admin/password"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct AdminLogout;

impl Url for AdminLogout {
    fn postfix(&self) -> &str {
        "/admin/logout"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct AdminSession;

impl Url for AdminSession {
    fn postfix(&self) -> &str {
        "/admin/session"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct Articles;

impl Url for Articles {
    fn postfix(&self) -> &str {
        "/articles"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct AdminArticles;

impl Url for AdminArticles {
    fn postfix(&self) -> &str {
        "/admin/articles"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct AdminEndpointHits {
    pub grouped: AdminEndpointHitsGrouped,
}

impl Url for AdminEndpointHits {
    fn postfix(&self) -> &str {
        "/admin/endpoint_hits"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct AdminEndpointHitsGrouped;

impl Url for AdminEndpointHitsGrouped {
    fn postfix(&self) -> &str {
        "/admin/endpoint_hits/grouped"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}
