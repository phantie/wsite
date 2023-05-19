use crate::routes::imports::*;

#[tracing::instrument(name = "Getting all the subscribers", skip_all)]
pub async fn all_subs(
    State(state): State<AppState>,
    session: ReadableSession,
) -> ApiResult<Json<Vec<Subscription>>> {
    reject_anonymous_users(&session)?;

    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let subscriptions_contents = subscriptions_docs
        .into_iter()
        .map(|doc| doc.contents)
        .collect::<Vec<_>>();

    Ok(Json(subscriptions_contents))
}
