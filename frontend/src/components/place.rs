#![allow(non_upper_case_globals)]

use crate::components::imports::*;
#[allow(unused)]
use crate::components::Markdown;

pub struct Place {
    theme_ctx: ThemeCtxSub,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    Nothing,
}

impl Component for Place {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Place");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::Nothing => false,
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        #![allow(unused)]
        use futures::{SinkExt, StreamExt};
        use gloo_net::websocket::{futures::WebSocket, Message};

        let ws = WebSocket::open("ws://127.0.0.1:8000/api/users_online").unwrap();

        let (mut write, mut read) = ws.split();

        ctx.link().send_future(async move {
            // write
            //     .send(Message::Text(String::from("test")))
            //     .await
            //     .unwrap();
            console::log!("started websocket reading future");
            while let Some(msg) = read.next().await {
                console::log!(format!("1. {:?}", msg))
            }
            // console::log!("WebSocket Closed");

            Self::Message::Nothing
        });

        // console::log!("connected to WS");

        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;

        let header_style = css!(
            "
                text-align: center;
            "
        );

        let canvas_style = css!(
            "
                height: 100vh;
                background-color: #131b25;
            "
        );

        let cells = vec![1, 2, 3, 4];

        html! {
            <>
                <h1 class={ header_style }>{ "Place" }</h1>
                <div class={ canvas_style }>
                    { cells }
                </div>
            </>
        }
    }
}
