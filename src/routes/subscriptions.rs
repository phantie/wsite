use axum::{extract::Form, http::StatusCode};

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(_payload): Form<FormData>) -> StatusCode {
    StatusCode::OK
}
