use crate::routes::imports::*;

#[derive(Serialize)]
pub struct AdminSession {
    session: Option<AdminSessionInner>,
}

#[derive(Serialize)]
pub struct AdminSessionInner {
    user_id: u64,
    username: String,
}

#[axum_macros::debug_handler]
pub async fn admin_session(
    State(state): State<AppState>,
    session: ReadableSession,
) -> Json<AdminSession> {
    let session = match session.get("user_id") {
        None => None,
        Some(user_id) => {
            let user = User::get_async(user_id, &state.database.collections.users)
                .await
                .unwrap()
                .unwrap();

            let username = user.contents.username;

            Some(AdminSessionInner { user_id, username })
        }
    };

    Json::from(AdminSession { session })
}
