use axum::response::{IntoResponse, Redirect, Response};
use axum_sessions::extractors::WritableSession;
use common::static_routes::*;

pub async fn logout(mut session: WritableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => return Redirect::to(routes().root.login.get().complete()).into_response(),
        Some(_user_id) => {
            session.destroy();
            tracing::info!("User successfully logged out.");
            Redirect::to(routes().root.login.get().complete()).into_response()
        }
    }
}
