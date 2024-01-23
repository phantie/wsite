const TURN_ON_LIGHT_THEME: bool = false;

// Circle in the right corner with absolute position
pub struct ThemeToggle {
    theme_ctx: ThemeCtxSub,
}

pub enum ThemeToggleMsg {
    ThemeContextUpdate(ThemeCtx),
    ToggleTheme,
}

#[derive(Properties, PartialEq)]
pub struct ThemeToggleProps {
    #[prop_or_default]
    pub children: Children,
}

impl Component for ThemeToggle {
    type Message = ThemeToggleMsg;
    type Properties = ThemeToggleProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(move |_| Self::Message::ToggleTheme);

        let theme = self.theme_ctx.as_ref();
        let toggle_border_color = &theme.box_border_color;
        let toggle_style = css!(
            "
                user-select: none;
                position: absolute; right: 15px; top: 15px;
                outline: 5px solid ${toggle_border_color};
                height: 2em; width: 2em;
                border-radius: 100%;
                cursor: pointer;
                transition: opacity .2s ease-in;

                :hover {
                    opacity: 0.8;
                }
            ",
            toggle_border_color = toggle_border_color
        );

        html! {
            <div {onclick} class={ toggle_style }/>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::ToggleTheme => {
                self.theme_ctx
                    .set_theme::<Self>(theme_toggle(self.theme_ctx.as_ref().id));
                false
            }
        }
    }
}

fn theme_toggle(theme: Themes) -> Themes {
    match theme {
        Themes::Dark => Themes::Pastel,
        Themes::Pastel => {
            if TURN_ON_LIGHT_THEME {
                Themes::Light
            } else {
                Themes::Dark
            }
        }
        Themes::Light => Themes::Dark,
    }
}

use super::theme_ctx::imports::*;
use crate::components::imports::*;
