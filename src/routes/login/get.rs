#![allow(unused_imports)]
use axum::response::{IntoResponse, Response};
use axum::{
    extract::{rejection::TypedHeaderRejection, Query, TypedHeader},
    response::Html,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn login_form(jar: CookieJar) -> (CookieJar, Html<&'static str>) {
    let error_cookie = jar.get("_flash");

    let error_html: String = match error_cookie {
        None => "".into(),
        Some(cookie) => {
            let error = cookie.value();
            format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
        }
    };

    // Html(include_str!("login.html"))
    let html: &'static str = Box::leak(
        format!(
            r#"
    <!DOCTYPE html>
    <html lang="en">
    
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Login</title>
    </head>
    
    <body>
        {error_html}
        <form action="/login" method="post">
            <label>Username
                <input type="text" placeholder="Enter Username" name="username">
            </label>
            <label>Password
                <input type="password" placeholder="Enter Password" name="password">
            </label>
            <button type="submit">Login</button>
        </form>
    </body>
    
    </html>
    "#
        )
        .into_boxed_str(),
    );

    let jar = jar.remove(Cookie::named("_flash"));

    (jar, Html(html))
}
