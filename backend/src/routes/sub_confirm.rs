use crate::routes::imports::*;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn sub_confirm(
    State(state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> ApiResult<()> {
    // TODO implement error handling like in subscriptions.rs

    let subscriptions = &state.database.collections.subscriptions;

    let docs = subscriptions
        .view::<schema::SubscriptionByToken>()
        .with_key(&parameters.subscription_token)
        .query_with_collection_docs()
        .await?;

    let doc = docs.into_iter().next().ok_or(ApiError::EntryNotFound)?;

    let mut doc = doc.document.clone();
    doc.contents.status = "confirmed".to_owned();
    doc.update_async(subscriptions).await?;
    Ok(())
}
