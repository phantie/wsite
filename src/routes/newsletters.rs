use crate::database::*;
use crate::startup::AppState;
use axum::extract::{Json, State};
use axum::http::StatusCode;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

pub async fn publish_newsletter(
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> StatusCode {
    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let confirmed_subscriptions = subscriptions_docs
        .into_iter()
        .filter(|doc| doc.contents.status == "confirmed");

    for subscriber in confirmed_subscriptions {
        state
            .email_client
            .send_email(
                &subscriber.contents.email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .unwrap();
    }

    StatusCode::OK
}
