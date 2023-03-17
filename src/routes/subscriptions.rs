use crate::database::*;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::AppState;
use anyhow::Context;
use axum::{
    extract::{Form, Json, State},
    http::StatusCode,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

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

#[axum_macros::debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, state),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> Result<StatusCode, SubscribeError> {
    let new_subscriber: NewSubscriber = form.try_into().map_err(SubscribeError::ValidationError)?;

    let subscription_token = generate_subscription_token();

    send_confirmation_email(
        &state.email_client,
        &new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email")?;

    insert_subscriber(&state, &new_subscriber, subscription_token)
        .await
        .context("Failed to insert into the database")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
    Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, state)
)]
pub async fn insert_subscriber(
    state: &AppState,
    new_subscriber: &NewSubscriber,
    subscription_token: String,
) -> Result<CollectionDocument<Subscription>, bonsaidb::core::schema::InsertError<Subscription>> {
    Subscription {
        name: new_subscriber.name.as_ref().to_owned(),
        email: new_subscriber.email.clone(),
        status: "pending_confirmation".to_owned(),
        token: subscription_token,
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

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[derive(thiserror::Error, Debug)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for SubscribeError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        let (trace_message, status) = match &self {
            SubscribeError::ValidationError(_message) => {
                (self.to_string(), StatusCode::BAD_REQUEST)
            }
            SubscribeError::UnexpectedError(e) => (
                format!("{}: {}", &message, e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };
        tracing::error!("{}", trace_message);
        (status, message).into_response()
    }
}
