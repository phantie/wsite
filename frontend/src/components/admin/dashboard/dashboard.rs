use crate::components::imports::*;

use super::Logout;
use super::WelcomeMessage;

pub struct Dashboard {
    session: Option<interfacing::AdminSession>,
}

pub enum Msg {
    SetSession(interfacing::AdminSession),
    Unauthorized,
    SessionFetchFailure,
}

impl Component for Dashboard {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { session: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let username: Option<AttrValue> = self
            .session
            .clone()
            .and_then(|session| Some(session.username.into()));

        html! {
            <>
                <h1><WelcomeMessage {username}/></h1>
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
            Self::Message::SetSession(session) => {
                self.session = Some(session);
                true
            }
            Self::Message::Unauthorized => {
                console::log!(format!("message Unauthorized from Dashboard"));
                navigator
                    .push_with_query(
                        &Route::Login,
                        &HashMap::from([("error", "Login to access dashboard")]),
                    )
                    .unwrap();
                false
            }
            Self::Message::SessionFetchFailure => unimplemented!(),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_admin_session().await {
                    Ok(session) => Self::Message::SetSession(session),
                    Err(e) => match e {
                        FetchAdminSessionError::AuthError => Self::Message::Unauthorized,
                        _ => Self::Message::SessionFetchFailure,
                    },
                }
            });
        }
    }
}
