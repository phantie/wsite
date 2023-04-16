use crate::components::imports::*;

pub type SessionCtx = Rc<Option<interfacing::AdminSession>>;

pub struct WithSession {
    session: Session,
}

pub struct SessionCtxSub {
    session_ctx: SessionCtx,
    // keep handle for component rerender after a session is loaded
    _session_ctx_handle: ContextHandle<SessionCtx>,
}

impl AsRef<Option<interfacing::AdminSession>> for SessionCtxSub {
    fn as_ref(&self) -> &Option<interfacing::AdminSession> {
        &self.session_ctx
    }
}

impl SessionCtxSub {
    fn new(session_ctx: SessionCtx, _session_ctx_handle: ContextHandle<SessionCtx>) -> Self {
        Self {
            session_ctx,
            _session_ctx_handle,
        }
    }

    pub fn subscribe<COMP, F, M>(ctx: &Context<COMP>, f: F) -> Self
    where
        COMP: Component,
        M: Into<COMP::Message>,
        F: Fn(SessionCtx) -> M + 'static,
    {
        let (session_ctx, _session_ctx_handle) = ctx
            .link()
            .context(ctx.link().callback(f))
            .expect("Session context does not exist");

        Self::new(session_ctx, _session_ctx_handle)
    }

    pub fn set(&mut self, ctx: SessionCtx) {
        self.session_ctx = ctx;
    }
}

enum Session {
    Unloaded,
    Loaded(interfacing::AdminSession),
    Error(SessionError),
}

#[derive(Properties, PartialEq)]

pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[derive(Debug)]
pub enum Msg {
    SessionLoaded(interfacing::AdminSession),
    SessionError(SessionError),
}

impl Component for WithSession {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            session: Session::Unloaded,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.session {
            Session::Unloaded => {
                console::log!("drawing WithSession with Unloaded");

                let session = Rc::new(None);

                html! {
                    <ContextProvider<SessionCtx> context={session}>
                        { ctx.props().children.clone() }
                    </ContextProvider<SessionCtx>>
                }
            }
            Session::Loaded(session) => {
                console::log!("drawing WithSession with Loaded");
                let session = session.clone();
                let session = Rc::new(Some(session));

                html! {
                    <ContextProvider<SessionCtx> context={session}>
                        { ctx.props().children.clone() }
                    </ContextProvider<SessionCtx>>
                }
            }
            Session::Error(_) => return internal_problems(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        console::console_dbg!(&msg);
        let navigator = ctx.link().navigator().unwrap();
        match msg {
            Self::Message::SessionLoaded(session) => {
                self.session = Session::Loaded(session);
                true
            }
            Self::Message::SessionError(e @ SessionError::AuthError) => {
                self.session = Session::Error(e);
                console::log!(format!("message SessionMissing from Dashboard"));
                navigator
                    .push_with_query(
                        &Route::Login,
                        &HashMap::from([("error", "Login to access admin section")]),
                    )
                    .unwrap();
                false
            }
            Self::Message::SessionError(e) => {
                self.session = Session::Error(e);
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_admin_session().await {
                    Ok(session) => Self::Message::SessionLoaded(session),
                    Err(e) => Self::Message::SessionError(e),
                }
            });
        }
    }
}
