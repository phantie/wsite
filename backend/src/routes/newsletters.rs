use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    session: ReadableSession,
    Extension(db_client): Extension<SharedDbClient>,
    Json(newsletter): Json<Newsletter>,
) -> ApiResult<()> {
    let _user_id = reject_anonymous_users(&session)?;

    let subscriptions = &db_client.read().await.collections().subs;

    let confirmed_subscriptions = subscriptions
        .view::<schema::SubscriptionByStatus>()
        .with_key("confirmed")
        .query_with_collection_docs()
        .await?;

    for subscriber in &confirmed_subscriptions {
        state
            .email_client
            .send_email(
                &subscriber.document.contents.email,
                &newsletter.title,
                &newsletter.content.html,
                &newsletter.content.text,
            )
            .await
            .context("Failed to send email to a confirmed subscriber")?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct Newsletter {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
}
