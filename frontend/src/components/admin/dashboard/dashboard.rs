use crate::components::imports::*;

use super::Logout;
use super::WelcomeMessage;

pub struct Dashboard {
    session: Session,
}

enum Session {
    Unloaded,
    Exist(interfacing::AdminSession),
    Missing,
    Unreachable,
    Invalid,
}

impl Session {
    fn is_error(&self) -> bool {
        match self {
            Self::Unreachable | Self::Invalid => true,
            _ => false,
        }
    }
}

pub enum Msg {
    SessionLoaded(interfacing::AdminSession),
    SessionMissing,
    SessionUnreachable,
    SessionInvalid,
}

impl Component for Dashboard {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            session: Session::Unloaded,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if self.session.is_error() {
            return internal_problems();
        }

        let username: Option<AttrValue> = match &self.session {
            Session::Unloaded => None,
            Session::Exist(session) => Some(session.username.clone().into()),
            Session::Missing => unreachable!("because you are redirected to login"),
            Session::Unreachable | Session::Invalid => {
                unreachable!("because of self.session.is_error() guard")
            }
        };

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
            Self::Message::SessionLoaded(session) => {
                self.session = Session::Exist(session);
                true
            }
            Self::Message::SessionMissing => {
                self.session = Session::Missing;
                console::log!(format!("message SessionMissing from Dashboard"));
                navigator
                    .push_with_query(
                        &Route::Login,
                        &HashMap::from([("error", "Login to access dashboard")]),
                    )
                    .unwrap();
                false
            }
            Self::Message::SessionInvalid => {
                self.session = Session::Invalid;
                true
            }
            Self::Message::SessionUnreachable => {
                self.session = Session::Unreachable;
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_admin_session().await {
                    Ok(session) => Self::Message::SessionLoaded(session),
                    Err(e) => match e {
                        FetchSessionError::AuthError => Self::Message::SessionMissing,
                        FetchSessionError::RequestError(_) => Self::Message::SessionUnreachable,
                        FetchSessionError::ParseError(_) => Self::Message::SessionInvalid,
                    },
                }
            });
        }
    }
}
