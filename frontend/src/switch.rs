use crate::router::Route;

use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::*;

    match routes {
        Route::NotFound => html! { <ErrorMessage message="not found 404"/> },
        Route::Unauthorized => {
            html! { <ErrorMessage message="unauthorized 401"/> }
        }
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::Login => html! { <Login/> },
        Route::AdminDashboard => {
            html! { <admin::Dashboard/>}
        }
    }
}
