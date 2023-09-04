#![allow(non_upper_case_globals)]

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
                    text-decoration: none;
                }
                
                a:hover {
                    text-decoration: underline;
                }

                li {
                    margin-bottom: 10px;
                }
            "
        );

        html! {
            <DefaultStyling>
                <Global css={global_style}/>
                <PageTitle title={"Dashboard"}/>

                <h1><WelcomeMessage/></h1>

                <ul>
                    <li>
                        <Link<Route> to={ Route::MarkdownPreview }>{ "Markdown preview" }</Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={ Route::ArticleList }>{ "Articles" }</Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={ Route::CreateArticle }>{ "Create article" }</Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={ Route::PasswordChange }>{ "Change password" }</Link<Route>>
                    </li>
                    <br/>
                    <li>
                        <Logout/>
                    </li>
                </ul>
            </DefaultStyling>
        }
    }
}
