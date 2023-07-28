#![allow(non_upper_case_globals)]

use crate::components::imports::*;

pub struct Header {
    theme_ctx: ThemeCtxSub,
    online_ctx: OnlineCtxSub,
    online: i32,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    OnlineContextUpdate(OnlineCtx),
    OnlineChange(i32),
}

impl Component for Header {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused)]
    fn create(ctx: &Context<Self>) -> Self {
        let online_ctx = OnlineCtxSub::subscribe(ctx, Self::Message::OnlineContextUpdate);
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            online: *online_ctx.as_ref(),
            online_ctx,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Header");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::OnlineContextUpdate(online_ctx) => {
                console::log!("WithTheme context updated from Header");
                self.online_ctx.set(online_ctx.clone());
                ctx.link()
                    .send_message(Self::Message::OnlineChange(*online_ctx));
                true
            }
            Self::Message::OnlineChange(value) => {
                self.online = value;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        let wrapper_style = css!(
            "
                display: flex;
                align-items: center;
                justify-content: right;
                height: 4em;
                width: 100%;
                background-color: ${bg_color};
                border-bottom: 2px solid ${box_border_color};
            ",
            bg_color = bg_color,
            box_border_color = box_border_color,
        );

        let online_style = css!(
            "
                font-size: 150%;
                margin: 0 4em;
            "
        );

        html! {
            <div class={ wrapper_style }>
                <div class={ online_style }>{ self.online }{ " Online" }</div>
            </div>
        }
    }
}
