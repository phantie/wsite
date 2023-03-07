use crate::{database::UserDatabase, startup::AppState};
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
    State(mut user_database): State<UserDatabase>,
    Form(form): Form<FormData>,
) -> StatusCode {
    user_database.add_user(("null".to_owned(), form.name.to_owned()));
    StatusCode::OK
}
