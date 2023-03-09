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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, state),
    fields(
        // fields, such as request_id field propagate
        request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> StatusCode {
    let _document = insert_subscriber(&state, &form).await;

    if let Err(e) = _document {
        tracing::error!("Failed to execute query: {:?}", e);
    }

    StatusCode::OK
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, state)
)]
pub async fn insert_subscriber(
    state: &AppState,
    form: &FormData,
) -> Result<CollectionDocument<Subscription>, bonsaidb::core::schema::InsertError<Subscription>> {
    Subscription {
        name: form.name.clone(),
        email: form.email.clone(),
    }
    .push_into_async(&state.database.collections.subscriptions)
    .await
}

#[tracing::instrument(
    name = "Getting all the subscribers",
    skip(state),
    fields( 
        request_id = %uuid::Uuid::new_v4(),
    )
)]
pub async fn all_subscriptions(State(state): State<AppState>) -> Json<Vec<Subscription>> {
    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let res = subscriptions_docs
        .iter()
        .map(|doc| doc.contents.clone())
        .collect::<Vec<_>>();

    Json(res)
}
