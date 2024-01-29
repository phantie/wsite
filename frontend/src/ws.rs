pub fn prepare_relative_url(relative_url: &str) -> String {
    let location = web_sys::window().unwrap().location();

    let hostname = location.hostname().unwrap();

    let protocol = if location.protocol().unwrap() == "http:" {
        "ws:"
    } else {
        "wss:"
    };

    let port = {
        let port = location.port().unwrap();
        // Due to Trunk Websocket proxy not working,
        // when developing with frontend dev server, connect directly to backend
        if port == "9000" {
            "8000".into()
        } else {
            port
        }
    };

    let url = {
        let url =
            web_sys::Url::new(format!("ws://127.0.0.1:8000{}", relative_url).as_str()).unwrap();
        url.set_hostname(&hostname);
        url.set_port(&port);
        url.set_protocol(protocol);
        url
    };

    url.to_string().as_string().unwrap()
}

pub mod imports {
    pub use futures::{stream::SplitStream, Stream, StreamExt};
    pub use gloo_net::websocket::{futures::WebSocket, Message};
}
