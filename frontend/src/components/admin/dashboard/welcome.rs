use crate::components::imports::*;

pub struct WelcomeMessage;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub username: Option<AttrValue>,
}

impl Component for WelcomeMessage {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let username = &ctx.props().username;
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
