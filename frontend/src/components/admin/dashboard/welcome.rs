use crate::components::imports::*;

pub struct WelcomeMessage {
    username: Option<AttrValue>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub no_auth_cb: Callback<()>,
}

pub enum Msg {
    SetUsername(AttrValue),
    Unauthorized,
}

impl Component for WelcomeMessage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { username: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.username {
            None => html! { "Welcome to dashboard" },
            Some(username) => html! {
               <>
                    <Colored with="orange">{ username }</Colored>
                    { ", welcome to dashboard" }
               </>
            },
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::SetUsername(username) => {
                self.username = Some(username);
                true
            }
            Self::Message::Unauthorized => {
                console::log!("Unauthorized");
                ctx.props().no_auth_cb.emit(());
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_admin_session().await {
                    Ok(session) => Msg::SetUsername(AttrValue::from(session.username)),
                    Err(_e) => Msg::Unauthorized,
                }
            });
        }
    }
}
