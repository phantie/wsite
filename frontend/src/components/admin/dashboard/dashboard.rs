use crate::components::imports::*;

use super::WelcomeMessage;

#[styled_component]
pub fn AdminDashboard() -> Html {
    html! {
        <>
            <h1><WelcomeMessage/></h1>
            <p>{ "Available actions:" }</p>
            <ol>
                <li>
                    <a href={ routes().root.admin.password.get().complete().to_owned() }>{ "Change password" }</a>
                </li>
                <li>
                    <form name="logoutForm" action="/api/admin/logout" method="post">
                        <input type="submit" value="Logout"/>
                    </form>
                </li>
            </ol>
        </>
    }
}
