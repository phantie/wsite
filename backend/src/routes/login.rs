use crate::routes::imports::*;
use interfacing::LoginForm;

#[tracing::instrument(
    skip(maybe_form, session, shared_database),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn login(
    mut session: WritableSession,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    maybe_form: Result<Json<LoginForm>, JsonRejection>,
) -> Result<impl IntoResponse, LoginError> {
    let Json(form) = maybe_form?;
    let credentials: Credentials = form.into();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = HangingStrategy::default()
        .execute(
            |shared_database| async {
                let credentials = credentials.clone();
                validate_credentials(shared_database, &credentials)
                    .await
                    .map_err(|e| match e {
                        AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                        AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
                    })
                    .unwrap()
            },
            shared_database.clone(),
        )
        .await
        .unwrap();

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    session.regenerate();
    session
        .insert("user_id", user_id)
        .context("Failed to register user_id in a session")
        .map_err(LoginError::UnexpectedError)?;

    Ok(StatusCode::OK)
}

impl From<LoginForm> for Credentials {
    fn from(value: LoginForm) -> Self {
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
