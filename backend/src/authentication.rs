use crate::{
    database::*,
    error::{ApiError, ApiResult},
    telemetry::spawn_blocking_with_tracing,
};
use anyhow::Context;
use axum_sessions::extractors::ReadableSession;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

#[tracing::instrument(name = "Validate credentials", skip_all)]
pub async fn validate_credentials(
    db_client: SharedDbClient,
    credentials: &Credentials,
) -> ApiResult<u64> {
    let users = &db_client.read().await.collections().users;

    let mapped_users = users
        .view::<schema::UserByUsername>()
        .with_key(&credentials.username)
        .query_with_collection_docs()
        .await
        .unwrap();

    let user = mapped_users.into_iter().next();

    if let None = &user {
        tracing::info!(
            "trying to login as a nonexistent user: {}",
            &credentials.username
        );
    }

    let expected_password_hash = match &user {
        // even if user does not exist, take time to compare provided pwd with invalid
        None => common::auth::invalid_password_hash(),
        Some(doc) => doc.document.contents.password_hash.clone(),
    };

    let password = credentials.password.clone();

    spawn_blocking_with_tracing(move || {
        common::auth::verify_password_hash(
            expected_password_hash,
            password.expose_secret().as_bytes(),
        )
    })
    .await
    .context("failed to spawn a verify password hash task")
    .map_err(ApiError::UnexpectedError)?
    .context("invalid password")
    .map_err(ApiError::AuthError)?;

    Ok(user
        .expect("since password verified, user exists")
        .document
        .header
        .id)
}

pub fn reject_anonymous_users(session: &ReadableSession) -> ApiResult<u64> {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => Err(ApiError::AuthError(anyhow::anyhow!("User not logged in"))),
        Some(id) => Ok(id),
    }
}
