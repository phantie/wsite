use crate::components::imports::*;

use super::super::WithAuth;
use super::Logout;
use super::WelcomeMessage;

pub struct Dashboard;

impl Component for Dashboard {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        console::log!("drawing Dashboard");

        html! {
            <WithAuth>
                <h1><WelcomeMessage/></h1>
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
            </WithAuth>
        }
    }
}
