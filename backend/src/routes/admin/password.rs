use crate::cozo_db;
use crate::routes::imports::*;
use interfacing::PasswordChangeForm;

pub async fn change_password(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
    Json(form): Json<PasswordChangeForm>,
) -> ApiResult<impl IntoResponse> {
    let username = reject_anonymous_users(&session)?;

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        Err(ApiError::AuthError(anyhow::anyhow!(
            "You entered two different new passwords - the field values must match"
        )))?
    }

    let credentials = Credentials {
        username: username.clone(),
        password: form.current_password,
    };

    validate_credentials(db.clone(), &credentials).await?;

    let pwd_hash = auth::hash_pwd(form.new_password.expose_secret().as_bytes())?;

    cozo_db::queries::update_user_pwd_hash(&db, &username, &pwd_hash)?;

    tracing::info!("{}'s password has been changed", username);

    Ok(())
}
