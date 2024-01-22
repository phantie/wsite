// It works for theme, online counter, etc
// because the same root ctx component can change data.
//
// Components subscribing to the data can read, but not change it.
//
// TODO add feature for data flow both down and upstream
// UPD added, feature is experimental

use crate::components::imports::*;

pub mod imports {
    pub use super::{StateCtx, StateCtxSub, WithState};
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub secret: u16,
}

#[derive(derivative::Derivative)]
#[derivative(Clone, Debug, PartialEq)] // TODO remove Clone, Debug
pub struct _State {
    pub state: State,

    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    upstream_cb: Callback<State>,

    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    upstream_msg_cb: Callback<Msg>,
}

// experimental
#[allow(unused)]
impl _State {
    // modify state from children
    fn _upstream(&self) {
        self.upstream_cb.emit(self.state.clone());
    }

    fn upstream_msg(&self, msg: Msg) {
        self.upstream_msg_cb.emit(msg);
    }

    pub fn upstream<COMP: Component>(&self) {
        self.log_from::<COMP>();
        self._upstream();
    }

    // provides more logs, but less flexible.
    // all changes must be done in one go before upstreaming
    //
    // ! does not modify the variable it's called on
    // should not matter because the caller should reload after
    pub fn upstream_fn<COMP: Component, F>(&mut self, f: F)
    where
        F: FnOnce(State) -> State,
    {
        let state = f(self.state.clone());
        console::log!(format!(
            "{}\n\n  {:?}\n\t->\n  {:?}",
            std::any::type_name::<COMP>(),
            &self,
            &state,
        ));
        self.state = state.clone();
        self._upstream();
    }
}

#[allow(unused)]
impl _State {
    pub fn log(&self) {
        console::log!(format!("{:?}", self));
    }

    pub fn log_from<COMP: Component>(&self) {
        console::log!(format!(
            "{}\n\n  {:?}",
            std::any::type_name::<COMP>(),
            &self
        ));
    }
}

pub type StateCtx = Rc<_State>;

pub struct WithState {
    state: _State,
}

pub struct StateCtxSub {
    ctx: StateCtx,
    // keep handle for component rerender after a state is loaded
    _ctx_handle: ContextHandle<StateCtx>,
}

impl AsRef<_State> for StateCtxSub {
    fn as_ref(&self) -> &_State {
        &self.ctx
    }
}

impl StateCtxSub {
    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(StateCtx) -> M + 'static,
    {
        let (ctx, _ctx_handle) = ctx
            .link()
            .context(ctx.link().callback(f))
            .expect("_State context to exist");

        Self { ctx, _ctx_handle }
    }

    pub fn set(&mut self, ctx: StateCtx) {
        self.ctx = ctx;
    }
}

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub enum Msg {
    #[allow(unused)]
    StateChanged(State),
}

impl Component for WithState {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let upstream_cb = ctx.link().callback(Msg::StateChanged);
        let upstream_msg_cb = ctx.link().callback(|msg| msg);

        Self {
            state: _State {
                state: State { secret: 42 },
                upstream_msg_cb,
                upstream_cb,
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = Rc::new(self.state.clone());

        html! {
            <ContextProvider<StateCtx> context={state}>
                { ctx.props().children.clone() }
            </ContextProvider<StateCtx>>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::StateChanged(state) => {
                self.state.state = state;
                true
            }
        }
    }
}
