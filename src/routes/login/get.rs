#![allow(unused_imports)]
use axum::http::StatusCode;
use axum::response::Html;

pub async fn login_form() -> Html<&'static str> {
    Html(include_str!("login.html"))
}
