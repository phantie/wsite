use crate::routes::imports::*;

#[tracing::instrument(name = "Getting all the subscribers", skip(state, session))]
pub async fn all_subs(
    State(state): State<AppState>,
    session: ReadableSession,
) -> Json<Vec<Subscription>> {
    let _user_id: u64 = reject_anonymous_users(&session).unwrap();

    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let subscriptions_contents = subscriptions_docs
        .into_iter()
        .map(|doc| doc.contents)
        .collect::<Vec<_>>();

    Json(subscriptions_contents)
}
