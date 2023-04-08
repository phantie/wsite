use crate::routes::imports::*;

#[allow(dead_code)]
pub async fn home() -> Html<&'static str> {
    Html(include_str!("home.html"))
}
