use crate::database::*;
use axum::{extract::Form, http::StatusCode, Json};

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(Form(form): Form<FormData>) -> StatusCode {
    let storage = storage(false);

    let subscriptions_collection = storage
        .create_database::<Subscription>("users", true)
        .unwrap();

    let _document = Subscription {
        name: form.name,
        email: form.email,
    }
    .push_into(&subscriptions_collection)
    .expect("Should insert");

    StatusCode::OK
}

pub async fn all_subscriptions() -> Json<Vec<Subscription>> {
    let storage = storage(false);

    let subscriptions_collection = storage
        .create_database::<Subscription>("users", true)
        .unwrap();

    let subscriptions_docs = Subscription::all(&subscriptions_collection)
        .query()
        .expect("Should retrieve");

    dbg!(&subscriptions_docs);

    let res = subscriptions_docs
        .iter()
        .map(|doc| doc.contents.clone())
        .collect::<Vec<_>>();

    Json(res)
}
