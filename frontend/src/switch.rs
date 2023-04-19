use crate::router::Route;

use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::admin::WithSession;
    use crate::components::*;

    match routes {
        Route::NotFound => html! {<Colored with="red"><h1>{"not found 404"}</h1></Colored> },
        Route::Unauthorized => html! {<Colored with="red"><h1>{"unauthorized 401"}</h1></Colored> },
        Route::Home => html! {
            <WithTheme>
                <Markdown file={ include_str!("../md/home.md") } />
            </WithTheme>
        },
        Route::Login => html! { <WithTheme><Login/></WithTheme> },
        Route::AdminDashboard => {
            html! {<WithSession><admin::Dashboard/></WithSession>}
        }
        Route::PasswordChange => {
            html! {<WithSession><admin::PasswordChange/></WithSession>}
        }
    }
}
