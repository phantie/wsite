#[allow(unused_imports)]
use crate::primitives::{Get, Post, Url};
use macros::*;

#[derive(Default)]
pub struct Routes {
    pub health_check: HealthCheck,
    pub subs: Subscriptions,
    pub newsletters: Newsletters,
    pub login: Login,
    pub admin: Admin,
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
pub struct Subscriptions {
    pub confirm: SubscriptionsConfirm,
}

impl Url for Subscriptions {
    fn postfix(&self) -> &str {
        "/subscriptions"
    }

    fn prefix(&self) -> &str {
        "/api"
    }
}

#[derive(Default, Get)]
pub struct SubscriptionsConfirm;

impl Url for SubscriptionsConfirm {
    fn postfix(&self) -> &str {
        "/subscriptions/confirm"
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

#[derive(Default)]
pub struct Admin {
    pub password: AdminPassword,
    pub logout: AdminLogout,
    pub session: AdminSession,
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
