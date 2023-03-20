use crate::{database::*, startup::AppState};
use anyhow::Context;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    {extract::State, response::Redirect},
};
use axum_sessions::extractors::ReadableSession;

#[tracing::instrument(
    skip(state, session),
    fields(user_id=tracing::field::Empty)
)]
#[axum_macros::debug_handler]
pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: ReadableSession,
) -> Result<Response, DashboardError> {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => Ok(Redirect::to("/login").into_response()),
        Some(id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&id));
            let user = User::get_async(id, &state.database.collections.users)
                .await
                .context("Failed to fetch user by id")
                .map_err(DashboardError::UnexpectedError)?
                .context("No user found by the id")
                .map_err(DashboardError::AuthError)?;

            Ok((
                StatusCode::OK,
                format!("Welcome {}", user.contents.username),
            )
                .into_response())
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DashboardError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for DashboardError {
    fn into_response(self) -> axum::response::Response {
        let (trace_message, response) = match &self {
            Self::AuthError(_e) => (self.to_string(), Redirect::to("/login").into_response()),
            Self::UnexpectedError(e) => (
                format!("{}: {}", self.to_string(), e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ),
        };
        tracing::error!("{}", trace_message);
        response
    }
}