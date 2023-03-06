use crate::database::UserDatabase;
use axum::{
    extract::{Form, State},
    http::StatusCode,
};

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    Form(_payload): Form<FormData>,
    // State(state): State<UserDatabase>,
) -> StatusCode {
    StatusCode::OK
}
