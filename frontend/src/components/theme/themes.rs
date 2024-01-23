#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Themes {
    Dark,
    Light,
    Pastel,
}

impl Default for Themes {
    fn default() -> Self {
        Self::Dark
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
            contrast_bg_color: "#11171e",
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
            contrast_bg_color: "#efede6",
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
            contrast_bg_color: "#312b49",
            text_color: light,
            link_color: light,
            box_border_color: light,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: AttrValue,
    pub id: Themes,
    pub session_id: AttrValue,
    pub bg_color: AttrValue,
    pub contrast_bg_color: AttrValue,
    pub text_color: AttrValue,
    pub link_color: AttrValue,
    pub box_border_color: AttrValue,
}

struct RawTheme<'a> {
    pub name: &'a str,
    pub id: Themes,
    pub session_id: &'a str,
    pub bg_color: &'a str,
    pub contrast_bg_color: &'a str,
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
            contrast_bg_color: theme.contrast_bg_color.to_owned().into(),
            text_color: theme.text_color.to_owned().into(),
            link_color: theme.link_color.to_owned().into(),
            box_border_color: theme.box_border_color.to_owned().into(),
        }
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
        match LocalStorage::set(Self::SESSION_KEY, Theme::from(*self).session_id.to_string()) {
            Ok(()) => {}
            Err(_) => console::log!("failed to store theme in session storage"),
        }
    }
}

impl From<Themes> for Theme {
    fn from(value: Themes) -> Self {
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
            Theme::from(theme).session_id,
            value,
            "resulting theme's session_id must match with the provided value"
        );
        Ok(theme)
    }
}

use crate::components::imports::*;
