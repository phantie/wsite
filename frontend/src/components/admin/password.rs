use crate::components::imports::*;

use interfacing::PasswordChangeForm;

#[derive(Default, Clone)]
pub struct Refs {
    current_password_ref: NodeRef,
    new_password_ref: NodeRef,
    new_password_check_ref: NodeRef,
}

pub struct PasswordChange {
    refs: Refs,
}

pub enum Msg {
    PasswordChangeSuccess,
    PasswordChangeFailure { error: AttrValue },
}

impl Component for PasswordChange {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            refs: Refs::default(),
        }
    }

    #[allow(unused_variables)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let window = web_sys::window().unwrap();
        match msg {
            Self::Message::PasswordChangeSuccess => {
                window
                    .alert_with_message("You've changed your password")
                    .unwrap();
                false
            }
            Self::Message::PasswordChangeFailure { error } => {
                window.alert_with_message(&error).unwrap();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Refs {
            current_password_ref,
            new_password_ref,
            new_password_check_ref,
        } = self.refs.clone();

        let onsubmit = {
            let Refs {
                current_password_ref,
                new_password_ref,
                new_password_check_ref,
            } = self.refs.clone();

            ctx.link().callback_future(move |event: SubmitEvent| {
                event.prevent_default();

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

                async move {
                    console::log!(format!("submitting: {:?}", password_form));
                    let password_change_response =
                        request_password_change(&password_form).await.unwrap();
                    password_change_response.log_status();

                    match password_change_response.status() {
                        200 => Msg::PasswordChangeSuccess,
                        401 | 400 => Msg::PasswordChangeFailure {
                            error: password_change_response.text().await.unwrap().into(),
                        },
                        _ => unimplemented!(),
                    }
                }
            })
        };

        html! {
            <DefaultStyling>
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
            </DefaultStyling>
        }
    }
}

async fn request_password_change(password_form: &PasswordChangeForm) -> request::SendResult {
    Request::static_post(routes().api.admin.password)
        .json(&password_form)
        .unwrap()
        .send()
        .await
}
