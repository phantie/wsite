#![allow(non_upper_case_globals)]

use crate::components::imports::*;

use crate::static_articles::{StaticArticle, static_articles};

enum Article {
    Dynamic(interfacing::Article),
    Static(StaticArticle)
}

impl Article {
    pub fn title(&self) -> String {
        match self {
            Article::Dynamic(article) => article.title.to_owned(),
            Article::Static(article) => article.title.to_owned(),
        }
    }

    pub fn public_id(&self) -> String {
        match self {
            Article::Dynamic(article) => article.public_id.to_owned(),
            Article::Static(article) => article.public_id.to_owned(),
        }
    }
}

pub struct ArticleList {
    articles: Option<Vec<Article>>,
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            articles: None,
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            session_ctx: SessionCtxSub::subscribe(ctx, Msg::SessionContextUpdate),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();
        let session = self.session_ctx.as_ref();

        let contrast_bg_color = &theme.contrast_bg_color;
        let box_border_color = &theme.box_border_color;

        match &self.articles {
            None => html! { <DefaultStyling/> },
            Some(articles) => {
                let global_style = css!(
                    "
                        a {
                            text-decoration: none;
                            color: inherit;
                        }
                    "
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
                    background-color: ${contrast_bg_color};
                    ",
                    box_border_color = box_border_color,
                    contrast_bg_color = contrast_bg_color
                );

                let articles = articles
                    .iter()
                    .map(|article| {
                        let public_id = article.public_id().clone();
                        let article_node_ref = NodeRef::default();

                        let delete_button = match article {
                            Article::Static(_) => html! {},
                            Article::Dynamic(_article) => {
                                match session {
                                    None => html! {},
                                    Some(_session) => {

                                        let onclick = {
                                            let public_id = public_id.clone();
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
                                }

                            }
                        };


                        let edit_button = match article {
                            Article::Static(_) => html! {},
                            Article::Dynamic(_article) => {
                                match session {
                                    None => html! {},
                                    Some(_session) => {
                                        let navigator = ctx.link().navigator().unwrap();
                                        let public_id = public_id.clone();
                                        let onclick = Callback::from(move |_| {
                                            let navigator = navigator.clone();
                                            let public_id = public_id.clone();
                                            navigator.push(&Route::EditArticle { public_id });
                                        });
        
                                        html! {
                                            <button {onclick}>{ "Edit" }</button>
                                        }
                                    }
                                }

                            }
                        };

                        let draft = match article { 
                            Article::Static(_) => html!{},
                            Article::Dynamic(article) => {
                                match session {
                                    Some(_session) => {
                                        match article.draft {
                                            true => html!{ "draft" },
                                            false => html!{},
                                        }
                                    }
                                    None => html!{},
                                }
                            }
                    };

                        html! {
                            <div key={public_id.clone()} ref={article_node_ref} class={article_classes.clone()}>

                                <Link<Route> to={ Route::ArticleViewer { public_id: public_id.clone() } }>
                                    <h1>{ &article.title() }</h1>
                                </Link<Route>>

                                {draft}

                                {delete_button}
                                {edit_button}
                            </div>
                        }
                    })
                    .collect::<Html>();

                let title_classes = css!("text-align: center; margin-bottom: 30px;");

                html! {
                    <DefaultStyling>
                        <Global css={global_style}/>
                        <PageTitle title={"Articles"}/>

                        <h1 class={title_classes}>{"Articles"}</h1>

                        <div class={article_wrapper_classes}>
                            {articles}
                        </div>
                    </DefaultStyling>
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
            Self::Message::ArticlesLoaded(dyn_articles) => {
                let mut articles = vec![];

                articles.extend(static_articles().into_iter().map(Article::Static));
                articles.extend(dyn_articles.into_iter().map(Article::Dynamic));

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
