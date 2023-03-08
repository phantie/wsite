#![allow(unused_variables)]

use crate::database::*;
use crate::startup::AppState;
use axum::extract::State;
use axum::{extract::Form, http::StatusCode, Json};

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> StatusCode {
    let _document = Subscription {
        name: form.name,
        email: form.email,
    }
    .push_into_async(&state.database.collections.subscriptions)
    .await
    .unwrap();

    StatusCode::OK
}

pub async fn all_subscriptions(State(state): State<AppState>) -> Json<Vec<Subscription>> {
    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let res = subscriptions_docs
        .iter()
        .map(|doc| doc.contents.clone())
        .collect::<Vec<_>>();
    println!("get~/all_subscriptions");
    Json(res)
}
