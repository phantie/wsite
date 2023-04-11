use crate::router::Route;

use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::*;

    match routes {
        Route::NotFound => html! {<Colored with="red"><h1>{"not found 404"}</h1></Colored> },
        Route::Unauthorized => html! {<Colored with="red"><h1>{"unauthorized 401"}</h1></Colored> },
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::Login => html! { <Login/> },
        Route::AdminDashboard => {
            html! { <admin::Dashboard/>}
        }
        Route::PasswordChange => {
            html! { <admin::PasswordChange/>}
        }
    }
}
