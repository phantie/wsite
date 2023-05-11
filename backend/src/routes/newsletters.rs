use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    maybe_basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection>,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(body): Json<BodyData>,
) -> Result<(), ApiError> {
    let TypedHeader(basic_auth) = maybe_basic_auth.map_err(ApiError::AuthHeaderRejection)?;

    let credentials: Credentials = basic_auth.into();
    let _user_id = validate_credentials(shared_database.clone(), &credentials).await?;

    let subscriptions = &state.database.collections.subscriptions;

    let confirmed_subscriptions = subscriptions
        .view::<SubscriptionByStatus>()
        .with_key("confirmed".to_owned())
        .query_with_collection_docs()
        .await?;

    for subscriber in &confirmed_subscriptions {
        state
            .email_client
            .send_email(
                &subscriber.document.contents.email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .context("Failed to send email to a confirmed subscriber")?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
}
