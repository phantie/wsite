use crate::authentication::compute_password_hash;
use crate::routes::imports::*;
use interfacing::PasswordChangeForm;

pub async fn change_password(
    State(state): State<AppState>,
    session: ReadableSession,
    Json(form): Json<PasswordChangeForm>,
) -> Result<StatusCode, PasswordChangeError> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        None.context("You entered two different new passwords - the field values must match.")
            .map_err(PasswordChangeError::PasswordConfirmDifferent)?
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

    let _user_id = validate_credentials(&state, &credentials)
        .await
        .context("The current password is incorrect")
        .map_err(PasswordChangeError::AuthError)?;

    let password_hash = compute_password_hash(form.new_password).unwrap();
    user.contents.password_hash = password_hash.expose_secret().to_owned();
    user.update_async(&state.database.collections.users)
        .await
        .unwrap();

    tracing::info!("Your password has been changed.");

    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordChangeError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error("{0}")]
    PasswordConfirmDifferent(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for PasswordChangeError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        let (trace_message, response) = match &self {
            Self::AuthError(_e) => (self.to_string(), StatusCode::UNAUTHORIZED),
            Self::PasswordConfirmDifferent(_e) => (self.to_string(), StatusCode::BAD_REQUEST),
            Self::UnexpectedError(e) => (
                format!("{}: {}", self.to_string(), e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };
        tracing::error!("{}", trace_message);
        (response, message).into_response()
    }
}
