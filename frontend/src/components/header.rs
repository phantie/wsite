#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::Online;

pub struct Header {
    theme_ctx: ThemeCtxSub,
    online: i32,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    OnlineChange(i32),
}

impl Component for Header {
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
                console::log!("WithTheme context updated from Header");
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
        let box_border_color = &theme.box_border_color;

        let wrapper_style = css!(
            "
                display: flex;
                align-items: center;
                justify-content: right;
                height: 4.5em;
                width: 100vw;
                background-color: ${bg_color};
                border-bottom: 2px solid ${box_border_color};
            ",
            bg_color = bg_color,
            box_border_color = box_border_color,
        );

        let online_style = css!(
            "
                font-size: 150%;
                margin: 0 5em;
            "
        );

        html! {
            <>
                <Online onchange={on_online_change}/>
                <div class={ wrapper_style }>
                    <div class={ online_style }>{ self.online }{ " Online" }</div>
                </div>
            </>
        }
    }
}
