use crate::database::*;
use crate::startup::AppState;
use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::Context;
use argon2::Argon2;
use argon2::{PasswordHash, PasswordVerifier};
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{Json, State, TypedHeader};
use axum::headers::{authorization::Basic, Authorization};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[axum_macros::debug_handler]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    maybe_basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let TypedHeader(basic_auth) = maybe_basic_auth?;

    let credentials: Credentials = basic_auth.into();
    let _user_id = validate_credentials(&state, &credentials).await?;

    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .context("Failed to fetch subscriptions")
        .map_err(PublishError::UnexpectedError)?;

    let confirmed_subscriptions = subscriptions_docs
        .into_iter()
        .filter(|doc| doc.contents.status == "confirmed");

    for subscriber in confirmed_subscriptions {
        state
            .email_client
            .send_email(
                &subscriber.contents.email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .context("Failed to send email to a confirmed subscriber")
            .map_err(PublishError::UnexpectedError)?;
    }

    Ok(StatusCode::OK)
}

struct Credentials {
    username: String,
    password: Secret<String>,
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
async fn validate_credentials(
    state: &AppState,
    credentials: &Credentials,
) -> Result<u64, PublishError> {
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
        .map_err(PublishError::UnexpectedError)?;

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
    .map_err(PublishError::UnexpectedError)??;

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
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(PublishError::AuthError)
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Auth header is rejected")]
    AuthHeaderRejection(#[from] TypedHeaderRejection),

    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        let headers = [(header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)];

        let message = self.to_string();
        let (trace_message, response) = match self {
            Self::AuthHeaderRejection(_rejection) => {
                (message, (StatusCode::UNAUTHORIZED, headers).into_response())
            }
            Self::AuthError(_e) => (message, (StatusCode::UNAUTHORIZED, headers).into_response()),
            Self::UnexpectedError(e) => (
                format!("{}: {}", &message, e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ),
        };

        tracing::error!("{}", trace_message);
        response
    }
}
