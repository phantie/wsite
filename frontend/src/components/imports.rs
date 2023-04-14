pub use crate::components::Colored;
pub use crate::router::Route;
pub use static_routes::*;

pub use std::collections::HashMap;

pub use gloo_console as console;
pub use gloo_net::http::{Request, Response};
pub use serde::{Deserialize, Serialize};
pub use stylist::yew::{styled_component, Global};
pub use web_sys::HtmlInputElement;

pub use secrecy::{ExposeSecret, SecretString};
pub use stylist::{css, style, Style};
pub use yew::prelude::*;
pub use yew_router::prelude::*;

pub trait RequestExtend {
    fn static_get(static_path: impl Get) -> Self;
    fn static_post(static_path: impl Post) -> Self;
}

impl RequestExtend for Request {
    fn static_get(static_path: impl Get) -> Self {
        Request::get(static_path.get().complete())
    }

    fn static_post(static_path: impl Post) -> Self {
        Request::post(static_path.post().complete())
    }
}

pub fn console_log_status(response: &Response) {
    console::log!(format!("{} status {}", response.url(), response.status()));
}

pub mod request {
    pub type SendResult = std::result::Result<gloo_net::http::Response, gloo_net::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum FetchAdminSessionError {
    #[error("Request error")]
    RequestError(#[source] gloo_net::Error),

    #[error("Authentication failed")]
    AuthError,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub async fn fetch_admin_session() -> Result<interfacing::AdminSession, FetchAdminSessionError> {
    use anyhow::Context;

    let response: Response = Request::static_get(routes().api.admin.session)
        .send()
        .await
        .map_err(FetchAdminSessionError::RequestError)?;

    if response.status() == 401 {
        Err(FetchAdminSessionError::AuthError)?
    }

    Ok(response
        .json::<interfacing::AdminSession>()
        .await
        .context("Failed to parse admin session")
        .map_err(FetchAdminSessionError::UnexpectedError)?)
}
