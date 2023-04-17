use crate::components::imports::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: AttrValue,
    pub id: Themes,
    pub bg_color: AttrValue,
    pub text_color: AttrValue,
    pub input_border_color: AttrValue,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".into(),
            id: Themes::Dark,
            bg_color: "#1B2430".into(),
            text_color: "white".into(),
            input_border_color: "white".into(),
        }
    }

    pub fn light() -> Self {
        let dark = "#212529";
        Self {
            name: "Light".into(),
            id: Themes::Light,
            bg_color: "#FEFCF3".into(),
            text_color: dark.into(),
            input_border_color: dark.into(),
        }
    }

    pub fn pastel() -> Self {
        let white = "#F2F7A1";
        Self {
            name: "Pastel".into(),
            id: Themes::Pastel,
            bg_color: "#453C67".into(),
            text_color: white.into(),
            input_border_color: white.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Themes {
    Dark,
    Light,
    Pastel,
}

impl From<&Themes> for Theme {
    fn from(value: &Themes) -> Self {
        match value {
            Themes::Dark => Theme::dark(),
            Themes::Light => Theme::light(),
            Themes::Pastel => Theme::pastel(),
        }
    }
}

pub type ThemeCtx = Rc<Theme>;

pub struct WithTheme {
    theme: Themes,
}

pub struct ThemeCtxSub {
    ctx: ThemeCtx,
    _ctx_handle: ContextHandle<ThemeCtx>,
}

impl AsRef<Theme> for ThemeCtxSub {
    fn as_ref(&self) -> &Theme {
        &self.ctx
    }
}

#[allow(dead_code)]
impl ThemeCtxSub {
    fn new(ctx: ThemeCtx, _ctx_handle: ContextHandle<ThemeCtx>) -> Self {
        Self { ctx, _ctx_handle }
    }

    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(ThemeCtx) -> M + 'static,
    {
        let (ctx, _ctx_handle) = ctx
            .link()
            .context(ctx.link().callback(f))
            .expect("Theme context does not exist");

        Self::new(ctx, _ctx_handle)
    }

    pub fn set(&mut self, ctx: ThemeCtx) {
        self.ctx = ctx;
    }
}

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub enum Msg {
    ToggleTheme,
}

impl Component for WithTheme {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme: Themes::Pastel,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = Rc::new(Theme::from(&self.theme));

        let onclick = ctx.link().callback(|_| Self::Message::ToggleTheme);

        let _btn_text = format!("{} theme", theme.name);

        let toggle_border_color = theme.text_color.clone();

        html! {
            <ContextProvider<ThemeCtx> context={theme}>
                { ctx.props().children.clone() }

                <div {onclick} class={ css!("
                    position: absolute; right: 10px; top: 10px;
                    border: 5px solid ${toggle_border_color};
                    height: 2em; width: 2em;
                    border-radius: 100%;
                    cursor: pointer;
                ", toggle_border_color = toggle_border_color) }/>
            </ContextProvider<ThemeCtx>>
        }
    }

    #[allow(unused_variables)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ToggleTheme => {
                let new_theme = match self.theme {
                    Themes::Dark => Themes::Light,
                    Themes::Light => Themes::Pastel,
                    Themes::Pastel => Themes::Dark,
                };

                self.theme = new_theme;
                true
            }
        }
    }
}
