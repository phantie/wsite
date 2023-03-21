#![allow(unused_imports)]
use crate::{
    authentication::{compute_password_hash, validate_credentials, Credentials},
    database::*,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::AppState,
};
use axum::{
    extract::{rejection::FormRejection, Form, Json, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_sessions::extractors::ReadableSession;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    State(state): State<AppState>,
    jar: CookieJar,
    session: ReadableSession,
    Form(form): Form<FormData>,
) -> Response {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let jar = jar.add(Cookie::new(
            "_flash",
            "You entered two different new passwords - the field values must match.",
        ));

        return (jar, Redirect::to("/admin/password")).into_response();
    }

    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => return Redirect::to("/login").into_response(),
        Some(id) => {
            let mut user = User::get_async(id, &state.database.collections.users)
                .await
                .unwrap()
                .unwrap();

            let credentials = Credentials {
                username: user.contents.username.clone(),
                password: form.current_password,
            };

            match validate_credentials(&state, &credentials).await {
                Ok(_user_id) => {
                    let password_hash = compute_password_hash(form.new_password).unwrap();

                    user.contents.password_hash = password_hash.expose_secret().to_owned();
                    user.update_async(&state.database.collections.users)
                        .await
                        .unwrap();
                    let jar = jar.add(Cookie::new("_flash", "Your password has been changed."));

                    (jar, Redirect::to("/admin/password")).into_response()
                }
                Err(_e) => {
                    let jar = jar.add(Cookie::new("_flash", "The current password is incorrect."));

                    return (jar, Redirect::to("/admin/password")).into_response();
                }
            }
        }
    }
}
