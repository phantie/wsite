#![allow(unused_imports)]

use axum::{
    extract::{rejection::FormRejection, Form, Json, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_sessions::extractors::WritableSession;

pub async fn logout(jar: CookieJar, mut session: WritableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => return Redirect::to("/login").into_response(),
        Some(id) => {
            session.destroy();
            let jar = jar.add(Cookie::new("_flash", "You have successfully logged out."));
            (jar, Redirect::to("/login")).into_response()
        }
    }
}
