#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::Markdown;

pub struct Post {
    theme_ctx: ThemeCtxSub,
    md: AttrValue,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub md: AttrValue,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
}

impl Component for Post {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            md: ctx.props().md.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Post");
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let padding_top = "2em";
        let padding_botttom = "4em";
        let style = css!(
            "
                .markdown-body {
                    font-size: 150%;
                    width: 1200px;
                    max-width: 90vw;
                }

                display: flex;
                justify-content: center;
                background-color: ${bg_color};
                padding-top: ${padding_top};
                padding-bottom: ${padding_botttom};
            ",
            bg_color = bg_color,
            padding_top = padding_top,
            padding_botttom = padding_botttom
        );

        html! {
            <div class={ style }>
                <Markdown md={ self.md.clone() } />
            </div>
        }
    }
}
