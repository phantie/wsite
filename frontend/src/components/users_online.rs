fn msg_stream(r: futures::stream::SplitStream<WebSocket>) -> impl Stream<Item = Msg> {
    r.map(|i| match i {
        Ok(msg) => match msg {
            Message::Text(text) => {
                console::log!(text);
                // TODO parse message
                Msg::CountChanged(-1)
            }
            Message::Bytes(_) => unimplemented!(),
        },
        Err(_) => Msg::Nothing,
    })
}

type Count = i32;

pub struct UsersOnlineCount {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(Callback::noop())]
    pub oninput: Callback<Count>,
}

pub enum Msg {
    CountChanged(Count),
    Nothing,
}

impl Component for UsersOnlineCount {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let ws = WebSocket::open("ws://127.0.0.1:8000/api/users_online").unwrap();
        let (_write, read) = ws.split();
        ctx.link().send_stream(msg_stream(read));
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::CountChanged(value) => {
                ctx.props().oninput.emit(value);
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
