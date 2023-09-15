#![allow(unused)]
use crate::components::imports::*;

pub struct Error {
    theme_ctx: ThemeCtxSub,
}

pub enum ErrorMsg {
    ThemeContextUpdate(ThemeCtx),
}

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub msg: AttrValue,
    pub code: u32,
}

impl Component for Error {
    type Message = ErrorMsg;
    type Properties = ErrorProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let text_color = &theme.text_color;

        let global_style =
            css! {"background-color: ${bg_color};padding: 40px;", bg_color = bg_color};
        let ErrorProps { msg, code } = ctx.props();
        html! {
            <div class={css!{"font-size: 40px; color: ${text_color};", text_color = text_color}}>
                <Global css={global_style}/>
                <div class={css!{"color:#ff5050;"}}>{ code }</div>
                { msg }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }
}
