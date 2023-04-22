#[allow(unused_imports)]
use crate::primitives::{Get, Post, Url};
use macros::*;

#[derive(Default)]
pub struct Routes {
    pub home: Home,
    pub login: Login,
    pub admin: Admin,
    pub articles: Articles,
}

#[derive(Default)]
pub struct Admin {
    pub password: AdminPassword,
    pub dashboard: AdminDashboard,
    pub articles: AdminArticles,
}

#[derive(Default, Get)]
pub struct Home;

impl Url for Home {
    fn postfix(&self) -> &str {
        "/"
    }
}

#[derive(Default, Get)]
pub struct Login;

impl Url for Login {
    fn postfix(&self) -> &str {
        "/login"
    }
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

#[derive(Default, Get)]
pub struct AdminArticles;

impl Url for AdminArticles {
    fn postfix(&self) -> &str {
        "/admin/articles"
    }
}

// article listing
#[derive(Default, Get)]
pub struct Articles;

impl Url for Articles {
    fn postfix(&self) -> &str {
        "/articles"
    }
}
