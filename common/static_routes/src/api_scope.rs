#[allow(unused_imports)]
use crate::primitives::{Get, Post, Url};
use macros::*;

#[derive(Default)]
pub struct Routes {
    pub health_check: HealthCheck,
    pub subs: Subs,
    pub newsletters: Newsletters,
    pub login: Login,
    pub admin: Admin,
    pub articles: Articles,
}

#[derive(Default)]
pub struct Admin {
    pub password: AdminPassword,
    pub logout: AdminLogout,
    pub session: AdminSession,
    pub articles: AdminArticles,
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

#[derive(Default, Get)]
pub struct Subs {
    pub confirm: SubConfirm,
    pub new: SubNew,
}

impl Url for Subs {
    fn postfix(&self) -> &str {
        "/subs"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct SubConfirm;

impl Url for SubConfirm {
    fn postfix(&self) -> &str {
        "/subs/confirm"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct SubNew;

impl Url for SubNew {
    fn postfix(&self) -> &str {
        "/subs/new"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Post)]
pub struct Newsletters;

impl Url for Newsletters {
    fn postfix(&self) -> &str {
        "/newsletters"
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
