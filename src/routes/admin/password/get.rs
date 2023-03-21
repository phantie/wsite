use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_sessions::extractors::ReadableSession;
use hyper::StatusCode;

pub async fn change_password_form(jar: CookieJar, session: ReadableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => return Redirect::to("/login").into_response(),
        Some(_id) => (),
    }

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

    (StatusCode::OK, jar, Html(html)).into_response()

    // Html(include_str!("get.html")).into_response()
}
