#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::Online;

pub struct Place {
    theme_ctx: ThemeCtxSub,
    online: i32,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    OnlineChange(i32),
}

impl Component for Place {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            online: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Place");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::OnlineChange(value) => {
                self.online = value;
                true
            }
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_online_change = ctx.link().callback(Self::Message::OnlineChange);

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
                <Online onchange={on_online_change}/>
                <h1 class={ header_style }>{ "Online:" }{ self.online }</h1>
                <div class={ canvas_style }>
                    { cells }
                </div>
            </>
        }
    }
}
