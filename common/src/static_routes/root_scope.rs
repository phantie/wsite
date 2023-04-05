use route_macros::*;
#[allow(unused_imports)]
use route_primitives::{Get, Post, Url};

#[derive(Default)]
pub struct Routes {
    pub home: Home,
    pub subs: Subscriptions,
    pub login: Login,
    pub admin: Admin,
}

#[derive(Default, Get)]
pub struct Home;

impl Url for Home {
    fn postfix(&self) -> &str {
        "/"
    }
}

#[derive(Default, Get)]
pub struct Subscriptions;

impl Url for Subscriptions {
    fn postfix(&self) -> &str {
        "/subscriptions"
    }
}

#[derive(Default, Get)]
pub struct Login;

impl Url for Login {
    fn postfix(&self) -> &str {
        "/login"
    }
}

#[derive(Default)]
pub struct Admin {
    pub password: AdminPassword,
    pub dashboard: AdminDashboard,
}

#[derive(Default, Get)]
pub struct AdminPassword;

impl Url for AdminPassword {
    fn postfix(&self) -> &str {
        "/admin/password"
    }
}

#[derive(Default, Get)]
pub struct AdminDashboard;

impl Url for AdminDashboard {
    fn postfix(&self) -> &str {
        "/admin/dashboard"
    }
}
