// use std::time::Duration;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    startup::AppState,
};
use anyhow::Context;
#[allow(unused_imports)]
use axum::{
    extract::{rejection::FormRejection, Form, Query, State},
    http::header,
    response::{IntoResponse, Redirect, Response},
};
use axum_sessions::extractors::WritableSession;
use secrecy::Secret;

#[tracing::instrument(
    skip(maybe_form, state, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    mut session: WritableSession,
    maybe_form: Result<Form<FormData>, FormRejection>,
) -> Result<Response, LoginError> {
    let Form(form) = maybe_form?;
    let credentials: Credentials = form.into();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(&state, &credentials)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    session.regenerate();
    session
        .insert("user_id", user_id)
        .context("Failed to register user_id in a session")
        .map_err(LoginError::UnexpectedError)?;

    Ok(Redirect::to("/admin/dashboard").into_response())
}

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

impl From<FormData> for Credentials {
    fn from(value: FormData) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Form is rejected")]
    FormRejection(#[from] FormRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        let trace_message = match &self {
            Self::FormRejection(_rejection) => self.to_string(),
            Self::AuthError(_e) => self.to_string(),
            Self::UnexpectedError(e) => format!("{}: {}", self.to_string(), e.source().unwrap()),
        };
        tracing::error!("{}", trace_message);
        let redirect = Redirect::to("/login");
        redirect.into_response()
    }
}
