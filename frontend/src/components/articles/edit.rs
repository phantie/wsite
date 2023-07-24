#![allow(non_upper_case_globals)]

use crate::components::imports::*;
#[allow(unused_imports)]
use crate::components::{ArticleEditor, ArticleEditorMode};

pub struct EditArticle {
    article: Option<interfacing::Article>,
}

pub enum Msg {
    ArticleLoaded(interfacing::Article),
    Nothing,
}

#[derive(Properties, PartialEq)]

pub struct Props {
    pub public_id: String,
}

impl Component for EditArticle {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self { article: None }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.article {
            None => html! {<DefaultStyling/>},
            Some(article) => {
                html! {
                    <DefaultStyling>
                        <ArticleEditor mode={ArticleEditorMode::Edit(self.article.clone().unwrap())}/>
                    </DefaultStyling>
                }
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let public_id = ctx.props().public_id.clone().to_string();
            ctx.link().send_future(async move {
                match fetch_article(public_id.as_str()).await {
                    Ok(article) => Self::Message::ArticleLoaded(article),
                    Err(_) => Self::Message::Nothing,
                }
            });
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ArticleLoaded(article) => {
                self.article = Some(article);
                true
            }
            Self::Message::Nothing => false,
        }
    }
}

async fn fetch_article(public_id: &str) -> Result<interfacing::Article, ()> {
    // duplicate
    let result = Request::get(&format!("/api/articles/{}", public_id))
        .send()
        .await;

    match result {
        Err(_) => Err(()),
        Ok(response) => match response.status() {
            200 => Ok(response.json::<interfacing::Article>().await.unwrap()),
            _ => Err(()),
        },
    }
}
