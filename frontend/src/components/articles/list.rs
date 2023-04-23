use crate::components::imports::*;
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct ArticleList {
    articles: Option<Vec<interfacing::Article>>,
    theme_ctx: ThemeCtxSub,
}

pub enum Msg {
    ArticlesLoaded(Vec<interfacing::Article>),
    ThemeContextUpdate(ThemeCtx),
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
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

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

                        html! {
                            <div {onclick} class={article_classes.clone()}>
                                <h1>{ &article.title }</h1>
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
