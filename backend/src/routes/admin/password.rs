use crate::routes::imports::*;
use interfacing::PasswordChangeForm;

pub async fn change_password(
    session: ReadableSession,
    Extension(_cozo_db): Extension<cozo::DbInstance>,
    Extension(db_client): Extension<SharedDbClient>,
    Json(form): Json<PasswordChangeForm>,
) -> ApiResult<impl IntoResponse> {
    let user_id = reject_anonymous_users(&session)?;

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        Err(ApiError::AuthError(anyhow::anyhow!(
            "You entered two different new passwords - the field values must match"
        )))?
    }

    HangingStrategy::long_linear()
        .execute(
            |db_client| async {
                let form = form.clone();
                async move {
                    let users = &db_client.read().await.collections().users;
                    let mut user = schema::User::get_async(&user_id, users)
                        .await?
                        .context("dangling user in session")?;
                    let credentials = Credentials {
                        username: user.contents.username.clone(),
                        password: form.current_password,
                    };
                    let _user_id = validate_credentials(db_client.clone(), &credentials).await?;

                    let password_hash =
                        auth::hash_pwd(form.new_password.expose_secret().as_bytes())?;
                    user.contents.password_hash = password_hash;
                    user.update_async(&db_client.read().await.collections().users)
                        .await?;

                    tracing::info!("Admin password has been changed.");

                    Ok(())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}
