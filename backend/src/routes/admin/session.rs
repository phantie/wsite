use crate::routes::imports::*;
use interfacing::AdminSession;

#[axum_macros::debug_handler]
pub async fn admin_session(
    Extension(db_client): Extension<SharedDbClient>,
    session: ReadableSession,
) -> ApiResult<Json<AdminSession>> {
    std::thread::sleep(std::time::Duration::from_millis(100));

    let session = match session.get::<u64>("user_id") {
        None => Err(ApiError::AuthError(anyhow::anyhow!("Session missing")))?,
        Some(user_id) => {
            let user = HangingStrategy::default()
                .execute(
                    |db_client| async {
                        async move {
                            let user = schema::User::get_async(
                                &user_id,
                                &db_client.read().await.collections().users,
                            )
                            .await?
                            .context("dangling user_id in session")?;

                            ApiResult::<_>::Ok(user)
                        }
                        .await
                    },
                    db_client.clone(),
                )
                .await??;

            let username = user.contents.username;

            AdminSession { user_id, username }
        }
    };

    Ok(Json::from(session))
}
