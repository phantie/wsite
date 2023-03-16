use crate::database::*;
use crate::startup::AppState;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, parameters))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> StatusCode {
    // TODO optimize using views or anything
    // TODO implement error handling like in subscriptions.rs
    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let subscription = subscriptions_docs
        .into_iter()
        .find(|doc| doc.contents.token == parameters.subscription_token);

    match subscription {
        Some(doc) => {
            // confirm subscriber
            let new_doc = {
                let mut d = doc.contents;
                d.status = "confirmed".to_owned();
                d
            };
            let _doc = Subscription::overwrite_async(
                doc.header.id,
                new_doc,
                &state.database.collections.subscriptions,
            )
            .await
            .unwrap();
            StatusCode::OK
        }
        // non-existing token and therefore subscriber
        None => StatusCode::UNAUTHORIZED,
    }
}
