use crate::router::Route;
use crate::switch::switch;

use yew::prelude::*;
use yew_router::prelude::{BrowserRouter, Switch};

#[function_component(App)]
pub fn app() -> Html {
    // use crate::components::ThemeToggle;
    use crate::components::WithOnline;
    use crate::components::WithTheme;

    html! {
        <WithTheme>
            // <ThemeToggle/> // here
            <WithOnline>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </WithOnline>
        </WithTheme>
    }
}
