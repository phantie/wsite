use crate::components::imports::*;
use interfacing::LoginForm;

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

            let login_form = LoginForm {
                username,
                password: SecretString::new(password),
            };

            wasm_bindgen_futures::spawn_local(async move {
                console::log!(format!("submitting: {:?}", login_form));

                let login_response = request_login(&login_form).await.unwrap();

                console_log_status(&login_response);

                match login_response.status() {
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
            <Global css={css!("
                display: flex;
                justify-content: center;
            ")}/>

            <h1 class={css!{"padding-top: 20px; padding-bottom: 20px;"}}>{ "Login" }</h1>

            <div>
                <form class={css!("width: 450px; max-width: 90vw;")}{onsubmit} method="post">
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

async fn request_login(login_form: &LoginForm) -> request::SendResult {
    Request::static_post(routes().api.login)
        .json(&login_form)
        .unwrap()
        .send()
        .await
}
