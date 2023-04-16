use crate::components::admin::SessionCtx;
use crate::components::imports::*;

pub struct WelcomeMessage {
    session_ctx: SessionCtx,
    // keep handle for component rerender after a session is loaded
    _session_ctx_handle: ContextHandle<SessionCtx>,
}

pub enum Msg {
    AuthContextUpdate(SessionCtx),
}

impl Component for WelcomeMessage {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::AuthContextUpdate(session_ctx) => {
                console::log!("WithSession context updated from WelcomeMessage");
                self.session_ctx = session_ctx;
                true
            }
        }
    }

    fn create(ctx: &Context<Self>) -> Self {
        let (session_ctx, _session_ctx_handle) = ctx
            .link()
            .context(
                ctx.link()
                    .callback(|session_ctx: SessionCtx| Msg::AuthContextUpdate(session_ctx)),
            )
            .expect("Session context must exist");

        Self {
            session_ctx,
            _session_ctx_handle,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let session_ctx = &self.session_ctx;

        console::log!(format!("drawing Welcome with {:?}", &session_ctx));

        let username: Option<AttrValue> = match (**session_ctx).clone() {
            None => None,
            Some(session) => Some(session.username.clone().into()),
        };

        match username {
            None => html! { "Welcome to dashboard, loading session..." },
            Some(username) => html! {
               <>
                    <Colored with="orange">{ username }</Colored>
                    { ", welcome to dashboard" }
               </>
            },
        }
    }
}
