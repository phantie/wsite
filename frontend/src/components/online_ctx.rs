use crate::components::imports::*;

type Online = i32;

enum State {
    Default,
    Updated(Online),
}

pub struct WithOnline {
    state: State,
}

pub type OnlineCtx = Rc<Online>;

pub struct OnlineCtxSub {
    ctx: OnlineCtx,
    // keep handle for component rerender after it's loaded
    _ctx_handle: ContextHandle<OnlineCtx>,
}

impl AsRef<Online> for OnlineCtxSub {
    fn as_ref(&self) -> &Online {
        &self.ctx
    }
}

#[allow(unused)]
impl OnlineCtxSub {
    fn new(ctx: OnlineCtx, _ctx_handle: ContextHandle<OnlineCtx>) -> Self {
        Self { ctx, _ctx_handle }
    }

    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(OnlineCtx) -> M + 'static,
    {
        let (ctx, _ctx_handle) = ctx
            .link()
            .context(ctx.link().callback(f))
            .expect("State context does not exist");

        Self::new(ctx, _ctx_handle)
    }

    pub fn set(&mut self, ctx: OnlineCtx) {
        self.ctx = ctx;
    }
}

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[derive(Debug)]
pub enum Msg {
    OnlineChanged(Online),
}

impl Component for WithOnline {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: State::Default,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = match &self.state {
            State::Default => {
                console::log!("drawing WithOnline with Default");
                0
            }
            State::Updated(state) => {
                console::log!(format!("drawing WithOnline with Updated({state})"));
                *state
            }
        };

        let on_online_change = ctx.link().callback(Self::Message::OnlineChanged);

        html! {
            <>
                <crate::components::Online onchange={on_online_change}/>
                <ContextProvider<OnlineCtx> context={Rc::new(state)}>
                    { ctx.props().children.clone() }
                </ContextProvider<OnlineCtx>>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::OnlineChanged(state) => {
                self.state = State::Updated(state);
                true
            }
        }
    }
}
