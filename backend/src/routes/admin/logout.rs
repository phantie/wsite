use axum::response::{IntoResponse, Redirect, Response};
use axum_sessions::extractors::WritableSession;

pub async fn logout(mut session: WritableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => return Redirect::to("/login").into_response(),
        Some(_user_id) => {
            session.destroy();
            tracing::info!("User successfully logged out.");
            Redirect::to("/login").into_response()
        }
    }
}
