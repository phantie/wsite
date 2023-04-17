use crate::components::imports::*;
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

#[derive(Default, Clone)]
pub struct Refs {
    username_ref: NodeRef,
    password_ref: NodeRef,
}

pub struct Login {
    theme_ctx: ThemeCtxSub,
    refs: Refs,
}

pub enum Msg {
    AuthSuccess,
    AuthFailure,
    AlreadyAuthed,
    Nothing,
    ThemeContextUpdate(ThemeCtx),
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            refs: Refs::default(),
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
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
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Login");
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        console::log!(format!(
            "drawing Login with theme {:?}",
            self.theme_ctx.as_ref()
        ));

        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let text_color = &theme.text_color;
        let input_border_color = &theme.input_border_color;

        let btn_style = match theme.id {
            Themes::Dark => "btn-outline-light",
            Themes::Light => "btn-outline-dark",
            Themes::Pastel => "btn-outline-warning",
        };

        let btn_classes = classes!("btn", "btn-lg", btn_style);

        let input_text_color = match theme.id {
            Themes::Dark => "white",
            Themes::Light => "black",
            Themes::Pastel => "white",
        };

        let input_classes = classes!(
            "form-control",
            "form-control-lg",
            css!(
                "
                    color: ${input_text_color}!important;
                    border-top: none!important;
                    border-right: none!important;
                    border-left: none!important;
                    border-radius: 0!important;
                ",
                input_text_color = input_text_color
            )
        );

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
                    login_response.log_status();

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

        html! (
            <>
                <Global css={css!("

                    display: flex;
                    justify-content: center;
                    background-color: ${bg_color}!important;

                    body {
                        font-family: \"Trebuchet MS\";
                        background-color: ${bg_color};
                        color: ${text_color};
                    }

                    input {
                        background-color: transparent!important;
                        border-width: 3px!important;
                        border-color: ${input_border_color}!important;
                    }
                    
                    input::placeholder {
                        color: ${text_color}!important;
                    }

                    *:focus {
                        outline: none!important;
                        outline-style: none!important;
                        box-shadow: none!important;
                    }
                ",
                    bg_color = bg_color,
                    text_color = text_color,
                    input_border_color = input_border_color
                )} />

                <h1 class={ css!{"padding-top: 20px; padding-bottom: 20px;"} }>{ "Login" }</h1>

                { error_node }

                <div>
                    <form class={ css!{"width: 450px; max-width: 90vw;"} }{ onsubmit } method="post">
                        <div class="form-group">
                            <h4><label for="username_input">{ "Username" }</label></h4>
                            <input ref={username_ref} type="text"
                            name="username" id="username_input"
                            class={ input_classes.clone() }
                            required={true}/>
                        </div>

                        <div class="form-group">
                            <h4><label for="password_input">{ "Password" }</label></h4>
                            <input ref={password_ref} type="password"
                            name="password" id="password_input"
                            class={ input_classes }
                            required={true}/>
                        </div>

                        <br/>
                        <button type="submit" class={btn_classes} >{ "Login" }</button>
                    </form>
                </div>
            </>
        )
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
