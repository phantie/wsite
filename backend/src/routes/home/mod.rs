use crate::routes::imports::*;

pub async fn home() -> Html<&'static str> {
    Html(include_str!("home.html"))
}
