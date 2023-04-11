use crate::components::imports::*;
use interfacing::PasswordChangeForm;

#[styled_component]
pub fn PasswordChange() -> Html {
    let current_password_ref = use_node_ref();
    let new_password_ref = use_node_ref();
    let new_password_check_ref = use_node_ref();

    let navigator = use_navigator().unwrap();

    let onsubmit = {
        let current_password_ref = current_password_ref.clone();
        let new_password_ref = new_password_ref.clone();
        let new_password_check_ref = new_password_check_ref.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let window = web_sys::window().unwrap();
            let _navigator = navigator.clone();

            let current_password = current_password_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();
            let new_password = new_password_ref.cast::<HtmlInputElement>().unwrap().value();
            let new_password_check = new_password_check_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();

            let password_form = PasswordChangeForm {
                current_password: SecretString::new(current_password),
                new_password: SecretString::new(new_password),
                new_password_check: SecretString::new(new_password_check),
            };

            wasm_bindgen_futures::spawn_local(async move {
                console::log!(format!("submitting: {:?}", password_form));

                let password_change_response =
                    request_password_change(&password_form).await.unwrap();

                console_log_status(&password_change_response);

                match password_change_response.status() {
                    200 => {
                        window
                            .alert_with_message("You've changed your password")
                            .unwrap();
                    }
                    401 => {
                        window.alert_with_message("Unauthorized").unwrap();
                    }
                    400 => {
                        window
                            .alert_with_message("Failed to change the password")
                            .unwrap();
                    }
                    _ => unimplemented!(),
                };
            })
        })
    };

    html! {
        <form {onsubmit} method="post">
            <label>{ "Current password" }
                <input ref={current_password_ref} type="password" name="current_password"/>
            </label>
            <br/>
            <label>{ "New password" }
                <input ref={new_password_ref} type="password" name="new_password"/>
            </label>
            <br/>
            <label>{ "Confirm new password" }
                <input ref={new_password_check_ref} type="password" name="new_password_check"/>
            </label>
            <br/>
            <button type="submit">{ "Change password" }</button>
        </form>
    }
}

async fn request_password_change(password_form: &PasswordChangeForm) -> request::SendResult {
    Request::static_post(routes().api.admin.password)
        .json(&password_form)
        .unwrap()
        .send()
        .await
}
