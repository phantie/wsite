use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    database::*,
    startup::AppState,
};
use anyhow::Context;
use axum::{
    extract::{rejection::TypedHeaderRejection, Json, State, TypedHeader},
    headers::{authorization::Basic, Authorization},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

#[axum_macros::debug_handler]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    maybe_basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let TypedHeader(basic_auth) = maybe_basic_auth?;

    let credentials: Credentials = basic_auth.into();
    let _user_id = validate_credentials(&state, &credentials)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;

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

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Auth header rejected.")]
    AuthHeaderRejection(#[from] TypedHeaderRejection),

    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
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
