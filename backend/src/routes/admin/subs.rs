use crate::routes::imports::*;

#[tracing::instrument(name = "Getting all the subscribers", skip_all)]
pub async fn all_subs(
    Extension(db_client): Extension<SharedDbClient>,
    session: ReadableSession,
) -> ApiResult<Json<Vec<schema::Subscription>>> {
    reject_anonymous_users(&session)?;

    let subscriptions_docs =
        schema::Subscription::all_async(&db_client.read().await.collections().subs)
            .await
            .unwrap();

    let subscriptions_contents = subscriptions_docs
        .into_iter()
        .map(|doc| doc.contents)
        .collect::<Vec<_>>();

    Ok(Json(subscriptions_contents))
}
