#![allow(unused_variables)]

use crate::database::*;
use crate::startup::AppState;
use axum::extract::State;
use axum::{extract::Form, http::StatusCode, Json};

use bonsaidb::core::connection::{AsyncConnection, AsyncStorageConnection};
use bonsaidb::core::schema::SerializedCollection;
use bonsaidb::core::transaction::{Operation, Transaction};
use bonsaidb::local::config::{Builder, StorageConfiguration};
use bonsaidb::local::AsyncStorage;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> StatusCode {
    let storage = AsyncStorage::open(
        StorageConfiguration::new("my-db.bonsaidb")
            .with_schema::<Subscription>()
            .unwrap(),
    )
    .await
    .unwrap();

    let subscriptions_collection = storage
        .create_database::<Subscription>("users", true)
        .await
        .unwrap();

    let _document = Subscription {
        name: form.name,
        email: form.email,
    }
    .push_into_async(&subscriptions_collection)
    .await
    .unwrap();

    StatusCode::OK
}

use serde::{Deserialize, Serialize};

pub async fn all_subscriptions(State(state): State<AppState>) -> Json<Vec<Subscription>> {
    let storage = state.storage;

    let subscriptions_collection = storage
        .create_database::<Subscription>("users", true)
        .await
        .unwrap();

    let subscriptions_docs = Subscription::all_async(&subscriptions_collection)
        .await
        .unwrap();

    let res = subscriptions_docs
        .iter()
        .map(|doc| doc.contents.clone())
        .collect::<Vec<_>>();

    Json(res)
}
