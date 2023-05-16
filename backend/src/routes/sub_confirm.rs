use crate::routes::imports::*;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn sub_confirm(
    State(state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> StatusCode {
    // TODO implement error handling like in subscriptions.rs

    let subscriptions = &state.database.collections.subscriptions;

    let mapped_docs = subscriptions
        .view::<SubscriptionByToken>()
        .with_key(parameters.subscription_token.to_owned())
        .query_with_collection_docs()
        .await
        .unwrap();

    let subscription = mapped_docs.into_iter().next();

    match subscription {
        Some(mapped_doc) => {
            let mut doc = mapped_doc.document.clone();
            doc.contents.status = "confirmed".to_owned();
            doc.update_async(subscriptions).await.unwrap();
            StatusCode::OK
        }
        // non-existing token and therefore subscriber
        None => StatusCode::UNAUTHORIZED,
    }
}
