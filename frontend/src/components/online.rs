fn read_stream(stream: SplitStream<WebSocket>) -> impl Stream<Item = Msg> {
    stream.map(|i| match i {
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
        let url = crate::ws::prepare_relative_url("/ws/users_online");
        let ws = WebSocket::open(&url).unwrap();
        let (_write, read) = ws.split();
        ctx.link().send_stream(read_stream(read));
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
use crate::ws::imports::*;
