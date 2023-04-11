use crate::components::imports::*;

pub struct Logout;

impl Component for Logout {
    type Message = ();
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();

        let onclick = Callback::from(move |event: MouseEvent| {
            event.prevent_default();

            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let logout_response = request_logout().await.unwrap();

                console_log_status(&logout_response);

                match logout_response.status() {
                    200 => {
                        navigator.push(&Route::Login);
                    }
                    _ => unreachable!(),
                };
            })
        });

        html! {
            // <a {onclick}>{ "Logout" }</a>
            <button {onclick} type="button">{ "Logout" }</button>
        }
    }
}

async fn request_logout() -> request::SendResult {
    Request::static_post(routes().api.admin.logout).send().await
}
