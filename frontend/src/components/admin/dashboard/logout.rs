use crate::components::imports::*;

pub struct Logout;

pub enum Msg {
    LogoutSuccess,
}

impl Component for Logout {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback_future(move |event: MouseEvent| {
            event.prevent_default();

            async {
                let logout_response = request_logout().await.unwrap();
                logout_response.log_status();

                match logout_response.status() {
                    200 => Msg::LogoutSuccess,
                    _ => unimplemented!(),
                }
            }
        });

        html! {
            // <a {onclick}>{ "Logout" }</a>
            <button {onclick} type="button">{ "Logout" }</button>
        }
    }

    #[allow(unused_variables)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let navigator = ctx.link().navigator().unwrap();

        match msg {
            Self::Message::LogoutSuccess => {
                navigator.push(&Route::Login);
                false
            }
        }
    }
}

async fn request_logout() -> request::SendResult {
    Request::static_post(routes().api.admin.logout).send().await
}
