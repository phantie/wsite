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

#[derive(derivative::Derivative)]
#[derivative(Clone, PartialEq)] // TODO remove Clone
pub struct _State<S> {
    pub state: S,

    #[derivative(PartialEq = "ignore")]
    upstream_cb: Callback<S>,

    #[derivative(PartialEq = "ignore")]
    upstream_msg_cb: Callback<Msg<S>>,
}

// experimental
#[allow(unused)]
impl<S: Clone> _State<S> {
    // modify state from children
    // broadcast that state has changed
    fn _upstream(&self) {
        self.upstream_cb.emit(self.state.clone());
    }
}

#[allow(unused)]
impl<S> _State<S> {
    pub fn upstream_msg(&self, msg: Msg<S>) {
        self.upstream_msg_cb.emit(msg);
    }
}

impl<S: std::fmt::Debug + Clone> _State<S> {
    pub fn upstream<COMP: Component>(&self) {
        self.log_from::<COMP>();
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
            "{}\n\n Current state:\n {:?}",
            std::any::type_name::<COMP>(),
            &self.state
        ));
    }
}

pub type StateCtx<S> = _State<S>;

pub struct StateCtxSub<S: 'static + PartialEq + Clone> {
    pub ctx: StateCtx<S>,
    // keep handle for component rerender after a state is loaded
    _ctx_handle: ContextHandle<StateCtx<S>>,
}

impl<S: PartialEq + Clone> AsRef<S> for StateCtxSub<S> {
    fn as_ref(&self) -> &S {
        &self.ctx.state
    }
}

impl<S: PartialEq + Clone> StateCtxSub<S> {
    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(S) -> M + 'static,
    {
        let (ctx, _ctx_handle) = ctx
            .link()
            .context(
                ctx.link()
                    .callback(move |ctx: StateCtx<S>| f(ctx.state.clone())),
            )
            .expect("_State context to exist");

        Self { ctx, _ctx_handle }
    }

    pub fn set(&mut self, state: S) {
        self.ctx.state = state;
    }
}

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub enum Msg<S> {
    #[allow(unused)]
    StateChanged(S),
}

pub struct WithState<S> {
    state: _State<S>,
}

impl<S: 'static + PartialEq + Clone + StateDefault> Component for WithState<S> {
    type Message = Msg<S>;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let upstream_cb = ctx.link().callback(Msg::StateChanged);
        let upstream_msg_cb = ctx.link().callback(|msg| msg);

        Self {
            state: _State {
                state: S::default_state(),
                upstream_msg_cb,
                upstream_cb,
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.state.clone();

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

pub trait StateDefault {
    fn default_state() -> Self;
}
