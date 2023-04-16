use crate::components::imports::*;

pub struct WithAuth {
    session: Session,
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

impl Component for WithAuth {
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
                console::log!("drawing WithAuth with Unloaded");

                let session = Rc::new(interfacing::AdminSession {
                    user_id: 0,
                    username: "unknown".into(),
                });

                html! {
                    <ContextProvider<Rc<interfacing::AdminSession>> context={session}>
                        <h3>{ "Loading session..."}</h3>
                        { ctx.props().children.clone() }
                    </ContextProvider<Rc<interfacing::AdminSession>>>
                }
            }
            Session::Loaded(session) => {
                console::log!("drawing WithAuth with Loaded");
                let session = session.clone();
                let session = Rc::new(session);

                html! {
                    <ContextProvider<Rc<interfacing::AdminSession>> context={session}>
                        <h3>{ "Session is loaded"}</h3>
                        { ctx.props().children.clone() }
                    </ContextProvider<Rc<interfacing::AdminSession>>>
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
                        &HashMap::from([("error", "Login to access dashboard")]),
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
