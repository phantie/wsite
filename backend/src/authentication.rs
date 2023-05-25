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

    let expected_password_hash = match &user {
        // even if user does not exist, take time to compare provided pwd with invalid
        None => SecretString::new(
            "$argon2id$v=19$m=15000,t=1,p=1$\
            gZiV/M1gPc22ElAH/Jh1Hw$\
            CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                .to_string(),
        ),
        Some(doc) => doc.document.contents.password_hash.clone().into(),
    };

    let password = credentials.password.clone();

    spawn_blocking_with_tracing(|| verify_password_hash(expected_password_hash, password))
        .await
        .context("failed to spawn a verify password hash task")
        .map_err(ApiError::UnexpectedError)??;

    Ok(user
        .expect("since password verified, user exists")
        .document
        .header
        .id)
}

/// It's a slow operation, 10ms kind of slow.
fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> ApiResult<()> {
    use argon2::PasswordVerifier;

    let expected_password_hash = argon2::PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    argon2::Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(ApiError::AuthError)
}

pub fn reject_anonymous_users(session: &ReadableSession) -> ApiResult<u64> {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => Err(ApiError::AuthError(anyhow::anyhow!("User not logged in"))),
        Some(id) => Ok(id),
    }
}
