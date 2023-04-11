use crate::routes::imports::*;

#[derive(Serialize)]
pub struct AdminSession {
    session: AdminSessionInner,
}

#[derive(Serialize)]
pub struct AdminSessionInner {
    user_id: u64,
    username: String,
}

#[axum_macros::debug_handler]
pub async fn admin_session(State(state): State<AppState>, session: ReadableSession) -> Response {
    let session = match session.get("user_id") {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(user_id) => {
            let user = User::get_async(user_id, &state.database.collections.users)
                .await
                .unwrap()
                .unwrap();

            let username = user.contents.username;

            AdminSessionInner { user_id, username }
        }
    };

    Json::from(AdminSession { session }).into_response()
}
