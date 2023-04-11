use crate::components::imports::*;

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
        html! {
            <>
                <h1><WelcomeMessage/></h1>
                <p>{ "Available actions:" }</p>
                <ol>
                    <li>
                        <Link<Route> to={Route::PasswordChange}>{ "Change password" }</Link<Route>>
                    </li>
                    <br/>
                    <li>
                        <Logout/>
                    </li>
                </ol>
            </>
        }
    }
}
