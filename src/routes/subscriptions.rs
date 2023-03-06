use axum::{extract::Form, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct FormData {
    _email: String,
    _name: String,
}

pub async fn subscribe(Form(_payload): Form<FormData>) -> StatusCode {
    StatusCode::OK
}
