use crate::app::Route;
use static_routes::*;

use gloo_console as console;
use gloo_net::http::Request;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use stylist::yew::styled_component;
use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew_router::prelude::*;

pub mod login {

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct LoginForm {
        pub username: String,
        pub password: String,
    }

    #[styled_component]
    pub fn Login() -> Html {
        let username_ref = use_node_ref();
        let password_ref = use_node_ref();

        let navigator = use_navigator().unwrap();

        let onsubmit = {
            let username_ref = username_ref.clone();
            let password_ref = password_ref.clone();

            Callback::from(move |event: SubmitEvent| {
                event.prevent_default();
                let window = web_sys::window().unwrap();
                let navigator = navigator.clone();
                let username = username_ref.cast::<HtmlInputElement>().unwrap().value();
                let password = password_ref.cast::<HtmlInputElement>().unwrap().value();

                let login_form = LoginForm { username, password };

                wasm_bindgen_futures::spawn_local(async move {
                    console::log!(format!("submitting: {:?}", login_form));

                    let login_post_request = Request::static_post(routes().api.login)
                        .json(&login_form)
                        .unwrap()
                        .send()
                        .await
                        .unwrap();

                    console::log!(format!("submit status: {}", login_post_request.status()));

                    match login_post_request.status() {
                        200 => {
                            navigator.push(&Route::AdminDashboard);
                        }
                        401 => {
                            window.alert_with_message("Unauthorized").unwrap();
                        }
                        _ => unreachable!(),
                    };
                })
            })
        };

        html! {
            <>
                <form {onsubmit} method="post">
                    <label>{ "Username" }
                        <input ref={username_ref} type="text" placeholder="Enter Username" name="username"/>
                    </label>
                    <label>{ "Password" }
                        <input ref={password_ref} type="password" placeholder="Enter Password" name="password"/>
                    </label>
                    <button type="submit">{ "Login" }</button>
                </form>
            </>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub message: String,
}

#[styled_component]
pub fn ErrorMessage(props: &ErrorProps) -> Html {
    let message = &props.message;

    let css = css! {"
        color: rgb(248 83 20);
    "};

    html! {<h1 class={css}>{ message }</h1>}
}

trait RequestExtend {
    fn static_get(static_path: impl Get) -> Self;
    fn static_post(static_path: impl Post) -> Self;
}

impl RequestExtend for Request {
    fn static_get(static_path: impl Get) -> Self {
        Request::get(static_path.get().complete())
    }

    fn static_post(static_path: impl Post) -> Self {
        Request::post(static_path.post().complete())
    }
}

#[styled_component]
pub fn AdminDashboard() -> Html {
    // let login_post_request = Request::static_get(routes().api.admin.session)
    //     .send()
    //     .await
    //     .unwrap();
    pub mod welcome {

        use gloo_net::http::Response;

        use super::*;

        #[derive(Default)]
        pub struct WelcomeMessage {
            node_ref: NodeRef,
            username: Option<AttrValue>,
        }

        pub enum Msg {
            SetUsername(AttrValue),
        }

        enum FetchUsername {
            Username(String),
            Unauthorized,
        }

        async fn fetch_username() -> FetchUsername {
            #[derive(Serialize, Deserialize, Debug)]
            pub struct AdminSession {
                session: AdminSessionInner,
            }

            #[derive(Serialize, Deserialize, Debug)]
            pub struct AdminSessionInner {
                user_id: u64,
                username: String,
            }

            let response: Response = Request::static_get(routes().api.admin.session)
                .send()
                .await
                .unwrap();

            match response.status() {
                200 => {
                    let session = response.json::<AdminSession>().await.unwrap();

                    let username = session.session.username;

                    FetchUsername::Username(username)
                }
                401 => FetchUsername::Unauthorized,
                _ => unreachable!(),
            }
        }

        impl Component for WelcomeMessage {
            type Message = Msg;
            type Properties = ();

            fn create(_ctx: &Context<Self>) -> Self {
                Self::default()
            }

            fn view(&self, _ctx: &Context<Self>) -> Html {
                let message = match &self.username {
                    None => "Welcome to Admin Dashboard".to_owned(),
                    Some(username) => format!("{}, welcome to Admin Dashboard", username),
                };

                html! {
                    <div ref={self.node_ref.clone()}>{ message }</div>
                }
            }

            fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
                match msg {
                    Msg::SetUsername(username) => {
                        self.username = Some(username);
                        true
                    }
                }
            }

            fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
                console::log!("calling rendered");

                if first_render {
                    ctx.link().send_future(async {
                        let username = fetch_username().await;

                        match username {
                            FetchUsername::Username(username) => {
                                Msg::SetUsername(AttrValue::from(username))
                            }
                            // TODO send Unauthorized event to the parent component
                            _ => unimplemented!(),
                        }
                    });
                }
                console::console_dbg!(self.username);
            }
        }
    }

    html! {
        <>
            <h1><welcome::WelcomeMessage/></h1>
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
