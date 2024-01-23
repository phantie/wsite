pub mod imports {
    pub use super::{ThemeCtx, ThemeCtxSub, WithTheme};
}

impl StateDefault for State {
    fn default_state() -> Self {
        Rc::new(Themes::derived().into())
    }
}

type State = Rc<Theme>;
pub type ThemeCtx = State;
pub type WithTheme = WithState<State>;
pub type ThemeCtxSub = StateCtxSub<State>;

impl ThemeCtxSub {
    pub fn set_theme<COMP: Component>(&mut self, theme: Themes) {
        theme.remember();
        self.ctx.state = Rc::new(Theme::from(theme));
        self.ctx.upstream::<COMP>();
    }
}

use super::themes::{Theme, Themes};
use crate::components::imports::*;
use crate::components::state::imports::*;
