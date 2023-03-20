use crate::{database::*, startup::AppState};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    {extract::State, response::Redirect},
};
use axum_sessions::extractors::ReadableSession;

pub async fn admin_dashboard(State(state): State<AppState>, session: ReadableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => Redirect::to("/login").into_response(),
        Some(id) => {
            let user = User::get_async(id, &state.database.collections.users)
                .await
                .unwrap()
                .unwrap();

            (
                StatusCode::OK,
                format!("Welcome {}", user.contents.username),
            )
                .into_response()
        }
    }
}
