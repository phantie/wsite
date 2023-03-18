use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::startup::AppState;
use axum::{
    extract::{rejection::FormRejection, Form, State},
    http::StatusCode,
    response::Redirect,
};
use secrecy::Secret;

#[tracing::instrument(
    skip(maybe_form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    maybe_form: Result<Form<FormData>, FormRejection>,
) -> Result<Redirect, LoginError> {
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

    Ok(Redirect::to("/"))
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
        let message = self.to_string();
        let (trace_message, status) = match &self {
            Self::FormRejection(_rejection) => (self.to_string(), StatusCode::BAD_REQUEST),
            Self::AuthError(_e) => (self.to_string(), StatusCode::UNAUTHORIZED),
            Self::UnexpectedError(e) => (
                format!("{}: {}", &message, e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };
        tracing::error!("{}", trace_message);
        (status, message).into_response()
    }
}
