use crate::routes::imports::*;

#[tracing::instrument(
    skip(maybe_form, state, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    mut session: WritableSession,
    maybe_form: Result<Json<FormData>, JsonRejection>,
) -> Result<impl IntoResponse, LoginError> {
    let Json(form) = maybe_form?;
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

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Clone)]
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
    FormRejection(#[from] JsonRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let trace_message = match &self {
            Self::FormRejection(rejection) => {
                format!("{}: {}", self.to_string(), rejection.to_string())
            }
            Self::AuthError(_e) => self.to_string(),
            Self::UnexpectedError(e) => format!("{}: {}", self.to_string(), e.source().unwrap()),
        };
        tracing::error!("{}", trace_message);
        StatusCode::UNAUTHORIZED.into_response()
    }
}
