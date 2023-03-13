use crate::database::*;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::startup::AppState;
use axum::extract::State;
use axum::{extract::Form, http::StatusCode, Json};

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, state),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> StatusCode {
    let new_subscriber: NewSubscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let result = insert_subscriber(&state, &new_subscriber).await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, state)
)]
pub async fn insert_subscriber(
    state: &AppState,
    new_subscriber: &NewSubscriber,
) -> Result<CollectionDocument<Subscription>, bonsaidb::core::schema::InsertError<Subscription>> {
    Subscription {
        name: new_subscriber.name.as_ref().to_owned(),
        email: new_subscriber.email.as_ref().to_owned(),
        status: "pending_confirmation".to_owned(),
        token: "NULL".to_owned(),
    }
    .push_into_async(&state.database.collections.subscriptions)
    .await
}

#[tracing::instrument(name = "Getting all the subscribers", skip(state))]
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
