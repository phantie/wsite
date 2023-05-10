use crate::routes::imports::*;
use interfacing::AdminSession;

#[axum_macros::debug_handler]
pub async fn admin_session(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    session: ReadableSession,
) -> Response {
    std::thread::sleep(std::time::Duration::from_millis(600));
    let session = match session.get("user_id") {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(user_id) => {
            let user = HangingStrategy::default()
                .execute(
                    |shared_database| async {
                        async move {
                            schema::User::get_async(
                                user_id,
                                &shared_database.read().await.collections.users,
                            )
                            .await
                            .unwrap()
                            // TODO if session exists and user does not - this panic
                            .unwrap()
                        }
                        .await
                    },
                    shared_database.clone(),
                )
                .await
                .unwrap();

            let username = user.contents.username;

            AdminSession { user_id, username }
        }
    };

    Json::from(session).into_response()
}
