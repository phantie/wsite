use crate::components::admin::dashboard::{Logout, WelcomeMessage};
use crate::components::imports::*;

pub struct Dashboard;

impl Component for Dashboard {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        console::log!("drawing Dashboard");

        let global_style = css!(
            "
                font-size: 150%;

                padding-left: 50px;

                a {
                    color: inherit;
                }
            "
        );

        html! {
            <DefaultStyling>
                <Global css={global_style}/>

                <h1><WelcomeMessage/></h1>
                <p>{ "Available actions:" }</p>
                <ol>
                    <li>
                        <Link<Route> to={ Route::ArticleEditor }>{ "Create article" }</Link<Route>>
                    </li>
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
            </DefaultStyling>
        }
    }
}
