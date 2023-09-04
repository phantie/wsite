use crate::routes::imports::*;
use interfacing::LoginForm;

#[tracing::instrument(
    skip_all,
    fields(username=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn login(
    mut session: WritableSession,
    Extension(db): Extension<cozo::DbInstance>,
    maybe_form: Result<Json<LoginForm>, JsonRejection>,
) -> ApiResult<()> {
    let Json(form) = maybe_form?;
    let credentials: Credentials = form.into();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let credentials = credentials.clone();

    validate_credentials(db, &credentials).await?;

    session.regenerate();

    session
        .insert("username", credentials.username)
        .context("Failed to register username in a session")?;

    Ok(())
}

impl From<LoginForm> for Credentials {
    fn from(value: LoginForm) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
