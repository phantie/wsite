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
pub struct _State<S> {
    pub state: S,

    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    upstream_cb: Callback<S>,

    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    upstream_msg_cb: Callback<Msg<S>>,
}

// experimental
#[allow(unused)]
impl<S: Clone> _State<S> {
    // modify state from children
    fn _upstream(&self) {
        self.upstream_cb.emit(self.state.clone());
    }
}

#[allow(unused)]
impl<S> _State<S> {
    fn upstream_msg(&self, msg: Msg<S>) {
        self.upstream_msg_cb.emit(msg);
    }
}

#[allow(unused)]
impl<S: std::fmt::Debug + Clone> _State<S> {
    pub fn upstream<COMP: Component>(&self) {
        self.log_from::<COMP>();
        self._upstream();
    }
}

#[allow(unused)]
impl<S: std::fmt::Debug + Clone> _State<S> {
    // provides more logs, but less flexible.
    // all changes must be done in one go before upstreaming
    //
    // ! does not modify the variable it's called on
    // should not matter because the caller should reload after
    pub fn upstream_fn<COMP: Component, F>(&mut self, f: F)
    where
        F: FnOnce(S) -> S,
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
impl<S: std::fmt::Debug> _State<S> {
    pub fn log(&self) {
        console::log!(format!("{:?}", self.state));
    }

    pub fn log_from<COMP: Component>(&self) {
        console::log!(format!(
            "{}\n\n  {:?}",
            std::any::type_name::<COMP>(),
            &self.state
        ));
    }
}

pub type StateCtx<S> = Rc<_State<S>>;

pub struct WithState<S> {
    state: _State<S>,
}

pub struct StateCtxSub<S: 'static + PartialEq> {
    ctx: StateCtx<S>,
    // keep handle for component rerender after a state is loaded
    _ctx_handle: ContextHandle<StateCtx<S>>,
}

impl<S: PartialEq> AsRef<_State<S>> for StateCtxSub<S> {
    fn as_ref(&self) -> &_State<S> {
        &self.ctx
    }
}

impl<S: PartialEq> StateCtxSub<S> {
    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(StateCtx<S>) -> M + 'static,
    {
        let (ctx, _ctx_handle) = ctx
            .link()
            .context(ctx.link().callback(f))
            .expect("_State context to exist");

        Self { ctx, _ctx_handle }
    }

    pub fn set(&mut self, ctx: StateCtx<S>) {
        self.ctx = ctx;
    }
}

#[derive(Properties, PartialEq)]

pub struct Props<S: PartialEq> {
    initial_state: S,

    #[prop_or_default]
    pub children: Children,
}

pub enum Msg<S> {
    #[allow(unused)]
    StateChanged(S),
}

impl<S: 'static + PartialEq + Clone> Component for WithState<S> {
    type Message = Msg<S>;
    type Properties = Props<S>;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let upstream_cb = ctx.link().callback(Msg::StateChanged);
        let upstream_msg_cb = ctx.link().callback(|msg| msg);

        Self {
            state: _State {
                state: ctx.props().initial_state.clone(),
                upstream_msg_cb,
                upstream_cb,
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = Rc::new(self.state.clone());

        html! {
            <ContextProvider<StateCtx<S>> context={state}>
                { ctx.props().children.clone() }
            </ContextProvider<StateCtx<S>>>
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

// type MyState = State<()>;
