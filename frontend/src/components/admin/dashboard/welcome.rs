use crate::components::imports::*;

type Session = Rc<interfacing::AdminSession>;

pub struct WelcomeMessage {
    session: Session,
    // keep handle for component rerender after a session is loaded
    _session_context_handle: ContextHandle<Session>,
}

pub enum Msg {
    AuthContextUpdate(Session),
}

impl Component for WelcomeMessage {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::AuthContextUpdate(session) => {
                console::log!("WithAuth context updated from WelcomeMessage");
                self.session = session;
                true
            }
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        let (session, _session_context_handle) = ctx
            .link()
            .context(
                ctx.link()
                    .callback(|session: Session| Msg::AuthContextUpdate(session)),
            )
            .unwrap();

        Self {
            session: session,
            _session_context_handle,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let session = &self.session;

        console::log!(format!("drawing Welcome with {:?}", &session));

        let username: Option<AttrValue> = Some(session.username.clone().into());

        match username {
            None => html! { "Welcome to dashboard" },
            Some(username) => html! {
               <>
                    <Colored with="orange">{ username }</Colored>
                    { ", welcome to dashboard" }
               </>
            },
        }
    }
}
