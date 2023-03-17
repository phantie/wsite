use crate::database::*;
use crate::startup::AppState;
use axum::body::{Bytes, Full};
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{Json, State, TypedHeader};
use axum::headers::{authorization::Basic, Authorization};
use axum::http::StatusCode;
use axum::response::Response;
use secrecy::Secret;

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
) -> Response<Full<Bytes>> {
    let basic_auth = match maybe_basic_auth {
        Ok(TypedHeader(basic_auth)) => basic_auth,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", r#"Basic realm="publish""#)
                .body(Full::from(""))
                .unwrap()
        }
    };
    let _credentials: Credentials = basic_auth.into();

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
        .body(Full::from(""))
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
