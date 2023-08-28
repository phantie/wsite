use crate::{
    database::*,
    error::{ApiError, ApiResult},
    telemetry::spawn_blocking_with_tracing,
    timeout::HangingStrategy,
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
    let user = HangingStrategy::long_linear()
        .execute(
            |db_client| async {
                async move {
                    let user = db_client
                        .read()
                        .await
                        .collections()
                        .user_by_username(&credentials.username)
                        .await?;
                    ApiResult::<_>::Ok(user)
                }
                .await
            },
            db_client.clone(),
        )
        .await??;

    if let None = &user {
        tracing::info!(
            "trying to login as a nonexistent user: {}",
            &credentials.username
        );
    }

    let expected_password_hash = match &user {
        // even if user does not exist, take time to compare provided pwd with invalid
        None => auth::invalid_password_hash(),
        Some(doc) => doc.contents.password_hash.clone(),
    };

    let password = credentials.password.clone();

    spawn_blocking_with_tracing(move || {
        auth::verify_password_hash(expected_password_hash, password.expose_secret().as_bytes())
    })
    .await
    .context("failed to spawn a verify password hash task")
    .map_err(ApiError::UnexpectedError)?
    .context("invalid password")
    .map_err(ApiError::AuthError)?;

    Ok(user
        .expect("since password verified, user exists")
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
