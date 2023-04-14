use crate::components::imports::*;

#[derive(Default, Clone)]
pub struct Refs {
    username_ref: NodeRef,
    password_ref: NodeRef,
}

pub struct Login {
    refs: Refs,
}

pub enum Msg {
    AuthSuccess,
    AuthFailure,
    AlreadyAuthed,
    Nothing,
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            refs: Refs::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let navigator = ctx.link().navigator().unwrap();
        match msg {
            Self::Message::AuthSuccess => {
                navigator.push(&Route::AdminDashboard);
                false
            }
            Self::Message::AuthFailure => {
                let window = web_sys::window().unwrap();
                let password_ref = self.refs.password_ref.clone();
                let password_field = password_ref.cast::<HtmlInputElement>().unwrap();
                password_field.set_value("");
                window.alert_with_message("Unauthorized").unwrap();
                true
            }
            Self::Message::AlreadyAuthed => {
                console::log!("already authed, redirect to dashboard");
                navigator.push(&Route::AdminDashboard);
                false
            }
            Self::Message::Nothing => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Refs {
            username_ref,
            password_ref,
        } = self.refs.clone();

        let onsubmit = {
            let Refs {
                username_ref,
                password_ref,
            } = self.refs.clone();

            ctx.link().callback_future(move |event: SubmitEvent| {
                event.prevent_default();

                let username_field = username_ref.cast::<HtmlInputElement>().unwrap();
                let password_field = password_ref.cast::<HtmlInputElement>().unwrap();

                let login_form = interfacing::LoginForm {
                    username: username_field.value(),
                    password: SecretString::new(password_field.value()),
                };

                async move {
                    console::log!(format!("submitting: {:?}", login_form));
                    let login_response = request_login(&login_form).await.unwrap();
                    console_log_status(&login_response);

                    match login_response.status() {
                        200 => Msg::AuthSuccess,
                        401 => Msg::AuthFailure,
                        _ => unimplemented!(),
                    }
                }
            })
        };

        let error_node = {
            let location = ctx.link().location().unwrap();
            let query_params = location.query::<HashMap<String, String>>().unwrap();
            match query_params.get("error") {
                None => html! {},
                Some(error) => html! {
                    <div class="alert alert-warning" role="alert">
                        { error }
                    </div>
                },
            }
        };

        html! {
            <>
                <Global css={ "display: flex; justify-content: center;" }/>

                <h1 class={ css!{"padding-top: 20px; padding-bottom: 20px;"} }>{ "Login" }</h1>

                { error_node }

                <div>
                    <form class={ css!{"width: 450px; max-width: 90vw;"} }{ onsubmit } method="post">
                        <div class="form-group">
                            <label class="col-form-label-lg" for="username_input">{ "Username" }</label>
                            <input ref={username_ref} type="text" placeholder="Enter Username"
                            name="username" id="username_input" class="form-control form-control-lg"
                            required={true}/>
                        </div>

                        <div class="form-group">
                            <label class="col-form-label-lg" for="password_input">{ "Password" }</label>
                            <input ref={password_ref} type="password" placeholder="Enter Password"
                            name="password" id="password_input" class="form-control form-control-lg"
                            required={true}/>
                        </div>

                        <button type="submit" class="btn btn-dark btn-lg">{ "Login" }</button>
                    </form>
                </div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_admin_session().await {
                    Ok(_session) => Self::Message::AlreadyAuthed,
                    Err(_e) => Self::Message::Nothing,
                }
            });
        }
    }
}

async fn request_login(login_form: &interfacing::LoginForm) -> request::SendResult {
    Request::static_post(routes().api.login)
        .json(&login_form)
        .unwrap()
        .send()
        .await
}
