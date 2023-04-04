use crate::authentication::reject_anonymous_users;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_sessions::extractors::ReadableSession;
use hyper::StatusCode;

pub async fn change_password_form(
    jar: CookieJar,
    session: ReadableSession,
) -> Result<Response, PasswordFormError> {
    let _user_id: u64 = reject_anonymous_users(&session).map_err(PasswordFormError::AuthError)?;

    let error_cookie = jar.get("_flash");

    let msg_html: String = match error_cookie {
        None => "".into(),
        Some(cookie) => {
            let error = cookie.value();
            format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
        }
    };
    let jar = jar.remove(Cookie::named("_flash"));

    let html: &'static str = Box::leak(
        format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">

            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Change Password</title>
            </head>

            <body>
                {msg_html}
                <form action="/admin/password" method="post">
                    <label>Current password
                        <input type="password" placeholder="Enter current password" name="current_password">
                    </label>
                    <br>
                    <label>New password
                        <input type="password" placeholder="Enter new password" name="new_password">
                    </label>
                    <br>
                    <label>Confirm new password
                        <input type="password" placeholder="Type the new password again" name="new_password_check">
                    </label>
                    <br>
                    <button type="submit">Change password</button>
                </form>
                <p><a href="/admin/dashboard">&lt;- Back</a></p>
            </body>

            </html>
        "#
        )
        .into_boxed_str(),
    );

    Ok((StatusCode::OK, jar, Html(html)).into_response())
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordFormError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for PasswordFormError {
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
