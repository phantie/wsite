use crate::components::imports::*;
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct DefaultStyling {
    theme_ctx: ThemeCtxSub,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
}

impl Component for DefaultStyling {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let text_color = &theme.text_color;

        let global_style = css!(
            "
                body {
                    background-color: ${bg_color};
                    color: ${text_color};
                }
            ",
            bg_color = bg_color,
            text_color = text_color,
        );

        html! {
            <>
                <Global css={global_style}/>
                { for ctx.props().children.iter() }
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Default background");
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }
}
