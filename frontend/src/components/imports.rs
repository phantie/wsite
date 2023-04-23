pub use crate::components::Colored;
pub use crate::router::Route;
pub use static_routes::*;

pub use std::collections::HashMap;
pub use std::rc::Rc;

pub use gloo_console as console;
pub use gloo_net::http::{Request, Response};
pub use serde::{Deserialize, Serialize};
pub use stylist::yew::{styled_component, Global};
pub use web_sys::{HtmlElement, HtmlInputElement};

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

pub trait ResponseExtend {
    fn log_status(&self);
}

impl ResponseExtend for Response {
    fn log_status(&self) {
        console::log!(format!("{} status {}", self.url(), self.status()));
    }
}

pub mod request {
    pub type SendResult = std::result::Result<gloo_net::http::Response, gloo_net::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("Authentication failed")]
    AuthError,

    #[error("Request error")]
    RequestError(#[source] gloo_net::Error),

    #[error("Bad status")]
    BadStatus(u16),

    #[error("Parsing error")]
    ParsingError(#[source] gloo_net::Error),
}

impl SessionError {
    #[allow(dead_code)]
    pub fn is_unexpected(&self) -> bool {
        match self {
            Self::AuthError => false,
            _ => true,
        }
    }
}

pub async fn fetch_admin_session() -> Result<interfacing::AdminSession, SessionError> {
    let response: Response = Request::static_get(routes().api.admin.session)
        .send()
        .await
        .map_err(SessionError::RequestError)?;

    match response.status() {
        200 => Ok(response
            .json::<interfacing::AdminSession>()
            .await
            .map_err(SessionError::ParsingError)?),
        401 => Err(SessionError::AuthError),
        status => Err(SessionError::BadStatus(status)),
    }
}

pub fn internal_problems() -> Html {
    html! {
        <>
            <Global css={ "display: flex; justify-content: center;" }/>

            <h1>{ "Ooops... internal problems" }</h1>
         </>
    }
}
