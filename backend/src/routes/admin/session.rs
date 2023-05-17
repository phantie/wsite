use crate::routes::imports::*;
use interfacing::AdminSession;

#[axum_macros::debug_handler]
pub async fn admin_session(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    session: ReadableSession,
) -> Result<Json<AdminSession>, ApiError> {
    std::thread::sleep(std::time::Duration::from_millis(100));

    let session = match session.get::<u64>("user_id") {
        None => Err(ApiError::AuthError(anyhow::anyhow!("Session missing")))?,
        Some(user_id) => {
            let user = HangingStrategy::default()
                .execute(
                    |shared_database| async {
                        async move {
                            let user = schema::User::get_async(
                                &user_id,
                                &shared_database.read().await.collections.users,
                            )
                            .await?
                            .context("dangling user_id in session")?;

                            Result::<_, ApiError>::Ok(user)
                        }
                        .await
                    },
                    shared_database.clone(),
                )
                .await??;

            let username = user.contents.username;

            AdminSession { user_id, username }
        }
    };

    Ok(Json::from(session))
}
