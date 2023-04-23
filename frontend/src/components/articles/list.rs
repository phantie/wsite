use crate::components::admin::{SessionCtx, SessionCtxSub};
use crate::components::imports::*;
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct ArticleList {
    articles: Option<Vec<interfacing::Article>>,
    theme_ctx: ThemeCtxSub,
    session_ctx: SessionCtxSub,
}

pub enum Msg {
    ArticlesLoaded(Vec<interfacing::Article>),
    ThemeContextUpdate(ThemeCtx),
    SessionContextUpdate(SessionCtx),
    ArticleRemoved(AttrValue),
    Nothing,
}

impl Component for ArticleList {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            articles: None,
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            session_ctx: SessionCtxSub::subscribe(ctx, Msg::SessionContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();
        let session = self.session_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let contrast_bg_color = &theme.contrast_bg_color;
        let text_color = &theme.text_color;
        let box_border_color = &theme.box_border_color;

        match &self.articles {
            None => html! {},
            Some(articles) => {
                let global_style = css!(
                    "
                        body {
                            background-color: ${bg_color};
                            color: ${text_color};
                        }
                    ",
                    bg_color = bg_color,
                    text_color = text_color,
                );

                let article_wrapper_classes = css!(
                    "
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                "
                );

                let article_classes = css!(
                    "
                    border: 2px solid ${box_border_color};
                    width: 800px;
                    max-width: 90vw;
                    margin-bottom: 20px;
                    padding: 15px 30px;
                    border-radius: 5px;
                    user-select: none;
                    cursor: pointer;
                ",
                    box_border_color = box_border_color
                );

                let navigator = ctx.link().navigator().unwrap();

                let articles = articles
                    .iter()
                    .map(|article| {
                        let onclick = {
                            let navigator = navigator.clone();
                            let public_id = article.public_id.clone();
                            Callback::from(move |_| {
                                navigator.push(&Route::ArticleViewer {
                                    public_id: public_id.clone(),
                                });
                            })
                        };
                        let article_node_ref = NodeRef::default();

                        let delete_button = match session {
                            None => html! {},
                            Some(_session) => {
                                let onclick = {
                                    let public_id = article.public_id.clone();
                                    let article_node_ref = article_node_ref.clone();

                                    ctx.link().callback_future(move |_| {
                                        let public_id = public_id.clone();
                                        let article_node_ref = article_node_ref.clone();

                                        async move {
                                            match delete_article(&public_id).await {
                                                Ok(_) => {
                                                    console::log!("article is removed");
                                                    article_node_ref
                                                        .clone()
                                                        .cast::<HtmlElement>()
                                                        .unwrap()
                                                        .remove();
                                                    Msg::ArticleRemoved(public_id.into())
                                                }
                                                Err(_) => {
                                                    console::log!("article is not removed");
                                                    Msg::Nothing
                                                }
                                            }
                                        }
                                    })
                                };

                                html! {
                                    <button { onclick }>{ "Delete" }</button>
                                }
                            }
                        };

                        html! {
                            <div key={article.public_id.clone()} ref={article_node_ref} class={article_classes.clone()}>
                                <h1 {onclick}>{ &article.title }</h1>
                                {delete_button.clone()}
                            </div>
                        }
                    })
                    .collect::<Html>();

                html! {
                    <>
                        <Global css={global_style}/>

                        <div class={article_wrapper_classes}>
                            {articles}
                        </div>
                    </>
                }
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_future(async {
                match fetch_article_list().await {
                    Ok(articles) => Self::Message::ArticlesLoaded(articles),
                    Err(_) => Self::Message::Nothing,
                }
            });
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ArticlesLoaded(articles) => {
                self.articles = Some(articles);
                true
            }
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Markdown Preview");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::SessionContextUpdate(session_ctx) => {
                console::log!("WithSession context updated from WelcomeMessage");
                self.session_ctx.set(session_ctx);
                true
            }
            Self::Message::ArticleRemoved(_public_id) => true,
            Self::Message::Nothing => false,
        }
    }
}

async fn fetch_article_list() -> Result<Vec<interfacing::Article>, ()> {
    let result = Request::static_get(routes().api.articles).send().await;

    match result {
        Err(_) => Err(()),
        Ok(response) => match response.status() {
            200 => Ok(response.json::<Vec<interfacing::Article>>().await.unwrap()),
            _ => Err(()),
        },
    }
}

async fn delete_article(public_id: &str) -> Result<(), ()> {
    let result = Request::delete(&format!("/api/articles/{}", public_id))
        .send()
        .await;

    match result {
        Err(_) => Err(()),
        Ok(response) => match response.status() {
            200 => Ok(()),
            _ => Err(()),
        },
    }
}
