use crate::{database::*, startup::AppState, telemetry::spawn_blocking_with_tracing};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::headers::{authorization::Basic, Authorization};
use secrecy::{ExposeSecret, Secret};

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl From<Authorization<Basic>> for Credentials {
    fn from(value: Authorization<Basic>) -> Self {
        Self {
            username: value.username().into(),
            password: Secret::new(value.password().into()),
        }
    }
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, state))]
pub async fn validate_credentials(
    state: &AppState,
    credentials: &Credentials,
) -> Result<u64, AuthError> {
    let mut user_id: Option<u64> = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    let user_docs = User::all_async(&state.database.collections.users)
        .await
        .context("Failed to fetch users")
        .map_err(AuthError::UnexpectedError)?;

    let user = user_docs
        .into_iter()
        .find(|doc| doc.contents.username == credentials.username);

    if let Some(doc) = user {
        user_id = Some(doc.header.id);
        expected_password_hash = Secret::new(doc.contents.password_hash);
    }

    let current_span = tracing::Span::current();
    let password = credentials.password.clone();

    // Tests that spawn an app run sequentially, therefore it does not speed up execution
    spawn_blocking_with_tracing(move || {
        current_span.in_scope(|| verify_password_hash(expected_password_hash, password))
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(AuthError::UnexpectedError)??;

    Ok(user_id.unwrap())
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
/// It's a slow operation, 10ms kind of slow.
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(AuthError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
