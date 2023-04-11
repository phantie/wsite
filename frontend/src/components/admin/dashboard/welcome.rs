use crate::components::imports::*;

pub struct WelcomeMessage {
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
        Self { username: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.username {
            None => html! { "Welcome to dashboard" },
            // Some(username) => format!("{}, welcome to dashboard", username),
            Some(username) => html! {
               <>
                   <div>
                       <Colored with="orange">{ username }</Colored>
                       { ", welcome to dashboard" }
                   </div>
               </>
            },
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
    }
}
