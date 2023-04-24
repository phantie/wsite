use crate::components::imports::*;
#[allow(unused_imports)]
use crate::components::{ArticleEditor, ArticleEditorMode};
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct EditArticle {
    theme_ctx: ThemeCtxSub,
    article: Option<interfacing::Article>,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
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
        Self {
            article: None,
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;

        let global_style = css!(
            "
                body {
                    background-color: ${bg_color};
                }
            ",
            bg_color = bg_color,
        );

        match &self.article {
            None => html! { <Global css={global_style}/> },
            Some(article) => {
                html! {
                    <ArticleEditor mode={ArticleEditorMode::Edit(self.article.clone().unwrap())}/>
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
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Markdown Preview");
                self.theme_ctx.set(theme_ctx);
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
