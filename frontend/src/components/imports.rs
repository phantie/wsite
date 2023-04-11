pub use crate::components::Colored;
pub use crate::router::Route;
pub use static_routes::*;

pub use gloo_console as console;
pub use gloo_net::http::{Request, Response};
pub use serde::{Deserialize, Serialize};
pub use stylist::yew::styled_component;
pub use web_sys::HtmlInputElement;

pub use secrecy::{ExposeSecret, SecretString};
pub use stylist::{style, Style};
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
