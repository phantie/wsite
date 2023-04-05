use crate::{
    authentication::reject_anonymous_users, database::*, startup::AppState, static_routes::*,
};
use anyhow::Context;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
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
    let id: u64 = reject_anonymous_users(&session).map_err(DashboardError::AuthError)?;

    tracing::Span::current().record("user_id", &tracing::field::display(&id));
    let user = User::get_async(id, &state.database.collections.users)
        .await
        .context("Failed to fetch user by id")
        .map_err(DashboardError::UnexpectedError)?
        .context("No user found by the id")
        .map_err(DashboardError::AuthError)?;

    let username = user.contents.username;

    let html: &'static str = Box::leak(
        format!(
            r#"
                <!DOCTYPE html>
                <html lang="en">
                
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Admin dashboard</title>
                </head>
                
                <body>
                    <p>Welcome {username}!</p>
                    <p>Available actions:</p>
                    <ol>
                        <li><a href="/admin/password">Change password</a></li>
                        <li>
                            <form name="logoutForm" action="/api/admin/logout" method="post">
                                <input type="submit" value="Logout">
                            </form>
                        </li>
                    </ol>
                </body>
                
                </html>
            "#
        )
        .into_boxed_str(),
    );

    Ok((StatusCode::OK, Html(html)).into_response())
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
            Self::AuthError(_e) => (
                self.to_string(),
                Redirect::to(routes().root.login.get().complete()).into_response(),
            ),
            Self::UnexpectedError(e) => (
                format!("{}: {}", self.to_string(), e.source().unwrap()),
                StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            ),
        };
        tracing::error!("{}", trace_message);
        response
    }
}
