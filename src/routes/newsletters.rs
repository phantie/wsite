use crate::database::*;
use crate::startup::AppState;
use crate::telemetry::spawn_blocking_with_tracing;
use argon2::Argon2;
use argon2::{PasswordHash, PasswordVerifier};
use axum::body::{Bytes, Empty};
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{Json, State, TypedHeader};
use axum::headers::{authorization::Basic, Authorization};
use axum::http::StatusCode;
use axum::response::Response;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[axum_macros::debug_handler]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    maybe_basic_auth: Result<TypedHeader<Authorization<Basic>>, TypedHeaderRejection>,
    Json(body): Json<BodyData>,
) -> Response<Empty<Bytes>> {
    let unauthorized = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", r#"Basic realm="publish""#)
        .body(Empty::new())
        .unwrap();

    let basic_auth = match maybe_basic_auth {
        Ok(TypedHeader(basic_auth)) => basic_auth,
        Err(_) => return unauthorized,
    };

    let credentials: Credentials = basic_auth.into();
    let user_id = validate_credentials(&state, &credentials).await;

    let _user_id = match user_id {
        None => return unauthorized,
        Some(user_id) => user_id,
    };

    let subscriptions_docs = Subscription::all_async(&state.database.collections.subscriptions)
        .await
        .unwrap();

    let confirmed_subscriptions = subscriptions_docs
        .into_iter()
        .filter(|doc| doc.contents.status == "confirmed");

    for subscriber in confirmed_subscriptions {
        state
            .email_client
            .send_email(
                &subscriber.contents.email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Empty::new())
        .unwrap()
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

impl From<Authorization<Basic>> for Credentials {
    fn from(value: Authorization<Basic>) -> Self {
        Self {
            username: value.username().into(),
            password: Secret::new(value.password().into()),
        }
    }
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, state))]
async fn validate_credentials(state: &AppState, credentials: &Credentials) -> Option<u64> {
    let user_docs = User::all_async(&state.database.collections.users)
        .await
        .unwrap();

    let user = user_docs
        .into_iter()
        .find(|doc| doc.contents.username == credentials.username);

    let (
        id,
        User {
            username: _username,
            password_hash: expected_password_hash,
        },
    ) = match user {
        Some(doc) => (doc.header.id, doc.contents),
        None => return None,
    };

    let current_span = tracing::Span::current();
    let password = credentials.password.expose_secret().clone();

    // Tests that spawn an app run sequentially, therefore it does not speed up execution
    spawn_blocking_with_tracing(move || {
        current_span.in_scope(|| {
            // It's a slow operation, 10ms kind of slow.
            Argon2::default().verify_password(
                password.as_bytes(),
                &PasswordHash::new(&expected_password_hash).ok().unwrap(),
            )
        })
    })
    .await
    .ok()?
    .ok()?;

    Some(id)
}
