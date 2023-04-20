use crate::components::imports::*;
use crate::components::Markdown;
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct Post {
    theme_ctx: ThemeCtxSub,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
}

impl Component for Post {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
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
        let style = css!(
            "
                background-color: ${bg_color};
                padding: 2em 4em;
            ",
            bg_color = bg_color,
        );

        html! {
            <div class={ style }>
                <Markdown file={ include_str!("../../md/home.md") } />
            </div>
        }
    }
}
