fn msg_stream(r: futures::stream::SplitStream<WebSocket>) -> impl Stream<Item = Msg> {
    r.map(|i| match i {
        Ok(msg) => match msg {
            Message::Text(text) => {
                console::log!(&text);
                match parse_online(&text) {
                    Ok(online) => Msg::OnlineChanged(online),
                    Err(_) => unimplemented!(),
                }
            }
            Message::Bytes(_) => unimplemented!(),
        },
        Err(_) => Msg::Nothing,
    })
}

fn parse_online(value: &str) -> Result<i32, anyhow::Error> {
    let r: Result<_, nom::Err<()>> = nom::bytes::complete::tag("users_online:")(value);
    let (num, _prefix) = r?;
    Ok(num.parse()?)
}

type Count = i32;

pub struct Online {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(Callback::noop())]
    pub onchange: Callback<Count>,
}

pub enum Msg {
    OnlineChanged(Count),
    Nothing,
}

impl Component for Online {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let location = web_sys::window().unwrap().location();
        let url = web_sys::Url::new("ws://127.0.0.1:8000/ws/users_online").unwrap();

        let hostname = location.hostname().unwrap();

        let protocol = location.protocol().unwrap();
        let protocol = if protocol == "http:" { "ws:" } else { "wss:" };

        let port = location.port().unwrap();
        // Due to Trunk Websocket proxy not working,
        // when developing with frontend dev server, connect directly to backend
        let port = if port == "9000" { "8000".into() } else { port };

        url.set_hostname(&hostname);
        url.set_port(&port);
        url.set_protocol(protocol);

        let ws = WebSocket::open(&url.to_string().as_string().unwrap()).unwrap();
        let (_write, read) = ws.split();
        ctx.link().send_stream(msg_stream(read));
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::OnlineChanged(value) => {
                ctx.props().onchange.emit(value);
                false
            }
            Self::Message::Nothing => false,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {}
    }
}

use crate::components::imports::*;
use futures::{Stream, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
