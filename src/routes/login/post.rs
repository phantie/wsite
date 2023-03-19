use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::configuration::get_configuration;
use crate::startup::AppState;
#[allow(unused_imports)]
use axum::{
    extract::{rejection::FormRejection, Form, Query, State},
    response::Redirect,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};

#[tracing::instrument(
    skip(maybe_form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
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
        let trace_message = match &self {
            Self::FormRejection(_rejection) => self.to_string(),
            Self::AuthError(_e) => self.to_string(),
            Self::UnexpectedError(e) => format!("{}: {}", self.to_string(), e.source().unwrap()),
        };
        tracing::error!("{}", trace_message);

        let query_string = format!("error={}", urlencoding::Encoded::new(self.to_string()));
        // Ideally to fetch this value from state
        // The downside can be noticed in testing
        let configuration = get_configuration();
        let secret: &[u8] = configuration
            .application
            .hmac_secret
            .expose_secret()
            .as_bytes();
        let hmac_tag = {
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
            mac.update(query_string.as_bytes());
            mac.finalize().into_bytes()
        };

        Redirect::to(&format!("/login?{query_string}&tag={hmac_tag:x}")).into_response()
    }
}
