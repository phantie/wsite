use crate::{
    authentication::{
        compute_password_hash, reject_anonymous_users, validate_credentials, Credentials,
    },
    database::*,
    startup::AppState,
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_sessions::extractors::ReadableSession;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    State(state): State<AppState>,
    session: ReadableSession,
    Form(form): Form<FormData>,
) -> Result<Response, PasswordChangeError> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        tracing::info!("You entered two different new passwords - the field values must match.");
        return Ok(Redirect::to("/admin/password").into_response());
    }

    let user_id: u64 = reject_anonymous_users(&session).map_err(PasswordChangeError::AuthError)?;

    let mut user = User::get_async(user_id, &state.database.collections.users)
        .await
        .unwrap()
        .unwrap();

    let credentials = Credentials {
        username: user.contents.username.clone(),
        password: form.current_password,
    };

    match validate_credentials(&state, &credentials).await {
        Ok(_user_id) => {
            let password_hash = compute_password_hash(form.new_password).unwrap();

            user.contents.password_hash = password_hash.expose_secret().to_owned();
            user.update_async(&state.database.collections.users)
                .await
                .unwrap();
            tracing::info!("Your password has been changed.");

            Ok(Redirect::to("/admin/password").into_response())
        }
        Err(_e) => {
            tracing::info!("The current password is incorrect.");

            return Ok(Redirect::to("/admin/password").into_response());
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordChangeError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for PasswordChangeError {
    fn into_response(self) -> axum::response::Response {
        let (trace_message, response) = match &self {
            Self::AuthError(_e) => (self.to_string(), Redirect::to("/login").into_response()),
            Self::UnexpectedError(e) => (
                format!("{}: {}", self.to_string(), e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ),
        };
        tracing::error!("{}", trace_message);
        response
    }
}
