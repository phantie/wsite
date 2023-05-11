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
) -> Result<(), ApiError> {
    let Json(form) = maybe_form?;
    let credentials: Credentials = form.into();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = HangingStrategy::default()
        .execute(
            |shared_database| async {
                let credentials = credentials.clone();
                let user_id = validate_credentials(shared_database, &credentials).await?;
                Result::<_, ApiError>::Ok(user_id)
            },
            shared_database.clone(),
        )
        .await??;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    session.regenerate();

    session
        .insert("user_id", user_id)
        .context("Failed to register user_id in a session")?;

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
