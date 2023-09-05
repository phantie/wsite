use crate::{
    db,
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
    db: cozo::DbInstance,
    credentials: &Credentials,
) -> ApiResult<()> {
    let user = db::q::find_user_by_username(&db, &credentials.username)?;

    if let None = &user {
        tracing::info!(
            "trying to login as a nonexistent user: {}",
            &credentials.username
        );
    }

    let expected_password_hash = match &user {
        // even if user does not exist, take time to compare provided pwd with invalid
        None => auth::invalid_password_hash(),
        Some(user) => user.pwd_hash.clone(),
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

    Ok(())
}

pub fn reject_anonymous_users(session: &ReadableSession) -> ApiResult<String> {
    let username: Option<String> = session.get("username");

    match username {
        None => Err(ApiError::AuthError(anyhow::anyhow!("User not logged in"))),
        Some(username) => Ok(username),
    }
}
