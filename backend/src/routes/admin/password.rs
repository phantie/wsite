use crate::authentication::compute_password_hash;
use crate::routes::imports::*;
use interfacing::PasswordChangeForm;

pub async fn change_password(
    session: ReadableSession,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(form): Json<PasswordChangeForm>,
) -> Result<impl IntoResponse, ApiError> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        Err(ApiError::AuthError(anyhow::anyhow!(
            "You entered two different new passwords - the field values must match"
        )))?
    }

    let user_id: u64 = reject_anonymous_users(&session)?;

    let _: Result<_, ApiError> = HangingStrategy::long_linear()
        .execute(
            |shared_database| async {
                let form = form.clone();
                async move {
                    let users = &shared_database.read().await.collections.users;
                    let mut user = schema::User::get_async(user_id, users)
                        .await?
                        .context("dangling user in session")?;
                    let credentials = Credentials {
                        username: user.contents.username.clone(),
                        password: form.current_password,
                    };
                    let _user_id =
                        validate_credentials(shared_database.clone(), &credentials).await?;

                    let password_hash = compute_password_hash(form.new_password)?;
                    user.contents.password_hash = password_hash.expose_secret().to_owned();
                    user.update_async(&shared_database.read().await.collections.users)
                        .await?;
                    Ok(())
                }
                .await
            },
            shared_database.clone(),
        )
        .await?;

    tracing::info!("Your password has been changed.");

    Ok(())
}
