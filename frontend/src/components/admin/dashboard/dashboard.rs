use crate::components::imports::*;

use super::Logout;
use super::WelcomeMessage;

pub struct Dashboard;

pub enum Msg {
    Unauthorized(AttrValue),
}

impl Component for Dashboard {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let no_auth_cb = |from: String| {
            ctx.link()
                .callback(move |()| Msg::Unauthorized(from.clone().into()))
        };

        html! {
            <>
                <h1><WelcomeMessage no_auth_cb={ no_auth_cb("WelcomeMessage".into()) }/></h1>
                <p>{ "Available actions:" }</p>
                <ol>
                    <li>
                        <Link<Route> to={ Route::PasswordChange }>{ "Change password" }</Link<Route>>
                    </li>
                    <li>
                        <a href={ routes().api.subs.get().complete().to_owned() }>{ "Subs" }</a>
                    </li>
                    <br/>
                    <li>
                        <Logout/>
                    </li>
                </ol>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let navigator = ctx.link().navigator().unwrap();
        match msg {
            Self::Message::Unauthorized(from) => {
                console::log!(format!("message Unauthorized from {} to Dashboard", from));
                navigator
                    .push_with_query(
                        &Route::Login,
                        &HashMap::from([("error", "Login to access dashboard")]),
                    )
                    .unwrap();
                false
            }
        }
    }
}
