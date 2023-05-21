use crate::{
    database::*,
    error::{ApiError, ApiResult},
    telemetry::spawn_blocking_with_tracing,
};
use anyhow::Context;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use axum::headers::{authorization::Basic, Authorization};
use axum_sessions::extractors::ReadableSession;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

impl From<Authorization<Basic>> for Credentials {
    fn from(value: Authorization<Basic>) -> Self {
        Self {
            username: value.username().into(),
            password: SecretString::new(value.password().into()),
        }
    }
}

#[tracing::instrument(
    name = "Validate credentials",
    skip_all,
    fields(
        success = tracing::field::Empty
    )
)]
pub async fn validate_credentials(
    db_client: SharedDbClient,
    credentials: &Credentials,
) -> ApiResult<u64> {
    let mut user_id: Option<u64> = None;
    let mut expected_password_hash = SecretString::new(
        "$argon2id$v=19$m=15000,t=1,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    let users = &db_client.read().await.collections().users;

    let mapped_users = users
        .view::<schema::UserByUsername>()
        .with_key(&credentials.username)
        .query_with_collection_docs()
        .await
        .unwrap();

    if let Some(doc) = mapped_users.into_iter().next() {
        user_id = Some(doc.document.header.id);
        expected_password_hash = SecretString::new(doc.document.contents.password_hash.clone());
    }

    let current_span = tracing::Span::current();
    let password = credentials.password.clone();

    // Tests that spawn an app run sequentially, therefore it does not speed up execution
    spawn_blocking_with_tracing(move || {
        current_span.in_scope(|| verify_password_hash(expected_password_hash, password))
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(ApiError::UnexpectedError)??;

    tracing::Span::current().record("success", &tracing::field::display(true));

    Ok(user_id.unwrap())
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
/// It's a slow operation, 10ms kind of slow.
fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> ApiResult<()> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(ApiError::AuthError)
}

pub fn compute_password_hash(password: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 1, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(SecretString::new(password_hash))
}

pub fn reject_anonymous_users(session: &ReadableSession) -> ApiResult<u64> {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => Err(ApiError::AuthError(anyhow::anyhow!("User not logged in"))),
        Some(id) => Ok(id),
    }
}
