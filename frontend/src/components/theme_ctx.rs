use crate::components::imports::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: AttrValue,
    pub id: Themes,
    pub session_id: AttrValue,
    pub bg_color: AttrValue,
    pub code_bg_color: AttrValue,
    pub text_color: AttrValue,
    pub link_color: AttrValue,
    pub box_border_color: AttrValue,
}

struct RawTheme<'a> {
    pub name: &'a str,
    pub id: Themes,
    pub session_id: &'a str,
    pub bg_color: &'a str,
    pub code_bg_color: &'a str,
    pub text_color: &'a str,
    pub link_color: &'a str,
    pub box_border_color: &'a str,
}

impl<'a> From<RawTheme<'a>> for Theme {
    fn from(theme: RawTheme) -> Self {
        Theme {
            name: theme.name.to_owned().into(),
            id: theme.id,
            session_id: theme.session_id.to_owned().into(),
            bg_color: theme.bg_color.to_owned().into(),
            code_bg_color: theme.code_bg_color.to_owned().into(),
            text_color: theme.text_color.to_owned().into(),
            link_color: theme.link_color.to_owned().into(),
            box_border_color: theme.box_border_color.to_owned().into(),
        }
    }
}

impl<'a> RawTheme<'a> {
    pub fn dark() -> Self {
        let light = "white";

        Self {
            name: "Dark",
            id: Themes::Dark,
            session_id: "dark",
            bg_color: "#1B2430",
            code_bg_color: "#11171e",
            text_color: light,
            link_color: light,
            box_border_color: light,
        }
    }

    pub fn light() -> Self {
        let dark = "#212529";
        Self {
            name: "Light",
            id: Themes::Light,
            session_id: "light",
            bg_color: "#FEFCF3",
            code_bg_color: "#efede6",
            text_color: dark,
            link_color: dark,
            box_border_color: dark,
        }
    }

    pub fn pastel() -> Self {
        let light = "#fffccd";
        Self {
            name: "Pastel",
            id: Themes::Pastel,
            session_id: "pastel",
            bg_color: "#453C67",
            code_bg_color: "#312b49",
            text_color: light,
            link_color: light,
            box_border_color: light,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Themes {
    Dark,
    Light,
    Pastel,
}

impl Default for Themes {
    fn default() -> Self {
        Self::Pastel
    }
}

impl Themes {
    const SESSION_KEY: &str = "theme";

    pub fn derived() -> Self {
        let remembered = || {
            use gloo_storage::{LocalStorage, Storage};
            LocalStorage::get::<String>(Self::SESSION_KEY)
        };

        let remembered_default = || {
            let theme = Self::default();
            theme.remember();
            theme
        };

        match remembered() {
            Ok(theme) => match Self::try_from(theme.as_str()) {
                Ok(theme) => theme,
                Err(_) => remembered_default(),
            },
            Err(_) => remembered_default(),
        }
    }

    pub fn remember(&self) {
        use gloo_storage::{LocalStorage, Storage};
        match LocalStorage::set(Self::SESSION_KEY, Theme::from(self).session_id.to_string()) {
            Ok(()) => {}
            Err(_) => console::log!("failed to store theme in session storage"),
        }
    }
}

impl From<&Themes> for Theme {
    fn from(value: &Themes) -> Self {
        match value {
            Themes::Dark => RawTheme::dark(),
            Themes::Light => RawTheme::light(),
            Themes::Pastel => RawTheme::pastel(),
        }
        .into()
    }
}

impl TryFrom<&str> for Themes {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let theme = match value {
            "dark" => Self::Dark,
            "light" => Self::Light,
            "pastel" => Self::Pastel,
            _ => return Err(()),
        };
        assert_eq!(
            Theme::from(&theme).session_id,
            value,
            "resulting theme's session_id must match with the provided value"
        );
        Ok(theme)
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
            theme: Themes::derived(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(move |_| Self::Message::ToggleTheme);

        let theme = Theme::from(&self.theme);
        let toggle_border_color = &theme.box_border_color;
        let toggle_style = css!(
            "
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
            <ContextProvider<ThemeCtx> context={ Rc::new(theme) }>
                { ctx.props().children.clone() }

                <div {onclick} class={ toggle_style }/>
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

                new_theme.remember();
                self.theme = new_theme;
                true
            }
        }
    }
}
