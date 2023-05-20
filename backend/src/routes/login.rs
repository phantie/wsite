use crate::routes::imports::*;
use interfacing::LoginForm;

#[tracing::instrument(
    skip_all,
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn login(
    mut session: WritableSession,
    Extension(db_client): Extension<SharedDbClient>,
    maybe_form: Result<Json<LoginForm>, JsonRejection>,
) -> ApiResult<()> {
    let Json(form) = maybe_form?;
    let credentials: Credentials = form.into();
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = HangingStrategy::default()
        .execute(
            |db_client| async {
                let credentials = credentials.clone();
                let user_id = validate_credentials(db_client, &credentials).await?;
                ApiResult::<_>::Ok(user_id)
            },
            db_client.clone(),
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
