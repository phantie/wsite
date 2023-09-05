#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::MarkdownPreview;

#[derive(PartialEq, Clone)]
pub enum ArticleEditorMode {
    Create,
    Edit(interfacing::ArticleWithId),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Article {
    New(interfacing::Article),
    Existing(interfacing::ArticleWithId),
}

impl Article {
    fn title(&self) -> String {
        match self {
            Self::New(article) => article.title.clone(),
            Self::Existing(article) => article.title.clone(),
        }
    }

    fn set_title(&mut self, value: String) {
        match self {
            Self::New(article) => article.title = value,
            Self::Existing(article) => article.title = value,
        }
    }

    fn markdown(&self) -> String {
        match self {
            Self::New(article) => article.markdown.clone(),
            Self::Existing(article) => article.markdown.clone(),
        }
    }

    fn set_markdown(&mut self, value: String) {
        match self {
            Self::New(article) => article.markdown = value,
            Self::Existing(article) => article.markdown = value,
        }
    }

    fn public_id(&self) -> String {
        match self {
            Self::New(article) => article.public_id.clone(),
            Self::Existing(article) => article.public_id.clone(),
        }
    }

    fn set_public_id(&mut self, value: String) {
        match self {
            Self::New(article) => article.public_id = value,
            Self::Existing(article) => article.public_id = value,
        }
    }

    fn draft(&self) -> bool {
        match self {
            Self::New(article) => article.draft.into(),
            Self::Existing(article) => article.draft.into(),
        }
    }

    fn set_draft(&mut self, value: bool) {
        match self {
            Self::New(article) => article.draft = value,
            Self::Existing(article) => article.draft = value,
        }
    }
}

pub struct ArticleEditor {
    theme_ctx: ThemeCtxSub,
    refs: Refs,
    mode: ArticleEditorMode,
    current_article_state: Article,
}

#[derive(Default, Clone)]
pub struct Refs {
    title_ref: NodeRef,
    public_id_ref: NodeRef,
    draft_ref: NodeRef,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    TitleChanged(String),
    PublicIDChanged(String),
    MarkdownChanged(AttrValue),
    DraftStateChanged(bool),
    NewArticleVersion(interfacing::ArticleWithId),
    Nothing,
}

#[derive(Properties, PartialEq)]

pub struct Props {
    pub mode: ArticleEditorMode,
}

impl Component for ArticleEditor {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let initial_article = match &ctx.props().mode {
            ArticleEditorMode::Create => Article::New(interfacing::Article {
                draft: true,
                ..Default::default()
            }),
            ArticleEditorMode::Edit(article) => Article::Existing(article.clone()),
        };

        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            refs: Refs::default(),
            mode: ctx.props().mode.clone(),
            current_article_state: initial_article,
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let contrast_bg_color = &theme.contrast_bg_color;
        let text_color = &theme.text_color;
        let box_border_color = &theme.box_border_color;

        let mode_display = match &self.mode {
            ArticleEditorMode::Create => "Create mode",
            ArticleEditorMode::Edit(_) => "Edit mode",
        };

        let action_classes = css!(
            "
            background-color: ${bg_color}; height: 50px; color: ${text_color};
            display: flex; align-items: center; padding: 0 20px; margin: 0 10px;
            cursor: pointer;
        ",
            bg_color = contrast_bg_color,
            text_color = text_color
        );

        let metadata_style = css!(
            "
            background-color: ${bg_color};
            display: flex;
            flex-direction: column;
            width: 350px;
            align-items: center;
            color: ${text_color};
        ",
            bg_color = bg_color,
            text_color = text_color
        );
        let metadata_classes = metadata_style;

        let metadatum_style = css!(
            "
            width: 85%;
            margin-bottom: 10px;

            label {
                font-size: 1.5em;
                font-weight: bold;
                margin-bottom: 10px;
                display: block;
            }

            input {
                width: inherit;
                background-color: transparent;
                border: 3px solid ${box_border_color};
                color: inherit;
                height: 30px;
                font-size: 150%;
                padding: 5px 15px;
                margin-bottom: 15px;
            }

            input::placeholder {
                color: ${input_text_color};
            }

            input:focus {
                outline-style: none;
            }
        ",
            input_text_color = text_color,
            box_border_color = box_border_color
        );

        let metadatum_classes = metadatum_style;

        let checkbox_classes = css!(
            "
            font-size: 150%;
            display: flex;
            align-items: baseline;
            margin-bottom: 30px;

            label {
                margin-right: 15px;
            }

            input {
                transform: scale(1.5);
            }
        "
        );

        let oninput = ctx.link().callback(Self::Message::MarkdownChanged);

        let actions_block = match &self.current_article_state {
            Article::New(article) => {
                let onclick = {
                    let article = article.clone();
                    let navigator = ctx.link().navigator().unwrap();

                    ctx.link().callback_future(move |_| {
                        let article = article.clone();
                        let navigator = navigator.clone();

                        async move {
                            console::log!(format!("submitting: {:?}", article));
                            let article = request_article_create(&article).await;

                            let window = web_sys::window().unwrap();
                            match article {
                                Ok(article) => {
                                    let navigator = navigator.clone();
                                    window.alert_with_message("Created!").unwrap();
                                    navigator.push(&Route::EditArticle {
                                        public_id: article.public_id,
                                    });
                                }
                                Err(_) => {
                                    window.alert_with_message("ERROR").unwrap();
                                }
                            }
                            Msg::Nothing
                        }
                    })
                };

                html! {
                    <div {onclick} class={action_classes.clone()}>{ "Save" }</div>
                }
            }
            Article::Existing(article) => {
                let onclick = {
                    let article = article.clone();

                    ctx.link().callback_future(move |_| {
                        let article = article.clone();

                        async move {
                            console::log!(format!("submitting: {:?}", article));
                            let r = request_article_update(&article).await.unwrap();
                            r.log_status();

                            let window = web_sys::window().unwrap();
                            match r.status() {
                                200 => {
                                    window.alert_with_message("Updated!").unwrap();
                                    Msg::NewArticleVersion(article)
                                }
                                _ => {
                                    window.alert_with_message("ERROR").unwrap();
                                    Msg::Nothing
                                }
                            }
                        }
                    })
                };

                html! {
                    <div {onclick} class={action_classes.clone()}>{ "Update" }</div>
                }
            }
        };

        let title_oninput = {
            let input_node_ref = self.refs.title_ref.clone();
            ctx.link().callback(move |_| {
                let input_field = input_node_ref.cast::<HtmlInputElement>().unwrap();
                let value = input_field.value();
                Self::Message::TitleChanged(value.into())
            })
        };

        let public_id_oninput = {
            let input_node_ref = self.refs.public_id_ref.clone();
            ctx.link().callback(move |_| {
                let input_field = input_node_ref.cast::<HtmlInputElement>().unwrap();
                let value = input_field.value();
                Self::Message::PublicIDChanged(value.into())
            })
        };

        let draft_oninput = {
            let input_node_ref = self.refs.draft_ref.clone();
            ctx.link().callback(move |_| {
                let input_field = input_node_ref.cast::<HtmlInputElement>().unwrap();
                let value = input_field.checked();
                Self::Message::DraftStateChanged(value)
            })
        };

        let title = match &self.mode {
            ArticleEditorMode::Create => html! {
                <PageTitle title={format!("New: {}", self.current_article_state.title())}/>
            },
            ArticleEditorMode::Edit(_) => html! {
                <PageTitle title={format!("Edit: {}", self.current_article_state.title())}/>
            },
        };

        html! {
            <DefaultStyling>
                { title }

                <div class={css!("display:flex;")}>
                    <div class={css!("height: 100vh; width: 100%;")}>
                        <MarkdownPreview {oninput} md={self.current_article_state.markdown() }/>
                    </div>

                    <div class={metadata_classes}>
                        <div class={css!{"height: 80px;"}}/>

                        <h2><div>{ mode_display }</div></h2>

                        <div class={metadatum_classes.clone()}>
                            <label for="title_input">{ "Title" }</label>
                            <input oninput={title_oninput} name="title_input"
                                ref={self.refs.title_ref.clone()}
                                value={ self.current_article_state.title() }
                            />
                        </div>
                        <div class={metadatum_classes.clone()}>
                            <label for="public_id_input">{ "Public ID" }</label>
                            <input oninput={public_id_oninput} name="public_id_input"
                                ref={self.refs.public_id_ref.clone()}
                                value={ self.current_article_state.public_id() }
                            />
                        </div>

                        <div class={checkbox_classes.clone()}>
                            <label for="draft_input">{ "Draft" }</label>
                            <input oninput={draft_oninput} name="draft_input" type="checkbox"
                                ref={self.refs.draft_ref.clone()}
                                checked={ self.current_article_state.draft() }
                            />
                        </div>

                        { actions_block }
                    </div>
                </div>
            </DefaultStyling>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from ArticleEditor");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::TitleChanged(value) => {
                console::log!(format!("title changed from ArticleEditor"));
                self.current_article_state.set_title(value);
                true
            }
            Self::Message::PublicIDChanged(value) => {
                console::log!(format!("public ID changed from ArticleEditor"));
                self.current_article_state.set_public_id(value);
                true
            }
            Self::Message::MarkdownChanged(value) => {
                console::log!(format!("markdown changed from ArticleEditor"));
                self.current_article_state.set_markdown(value.to_string());
                true
            }
            Self::Message::DraftStateChanged(value) => {
                console::log!(format!("draft state changed from ArticleEditor"));
                self.current_article_state.set_draft(value);
                true
            }
            Self::Message::NewArticleVersion(_value) => {
                console::log!("new article version saved from ArticleEditor");
                false
            }
            Self::Message::Nothing => false,
        }
    }
}

async fn request_article_create(
    article: &interfacing::Article,
) -> Result<interfacing::ArticleWithId, gloo_net::Error> {
    let response = Request::static_post(routes().api.admin.articles)
        .json(&article)
        .unwrap()
        .send()
        .await?;

    match response.status() {
        200 => Ok(response.json::<interfacing::ArticleWithId>().await?),
        _ => Err(gloo_net::Error::GlooError("status".into())),
    }
}

async fn request_article_update(article: &interfacing::ArticleWithId) -> request::SendResult {
    Request::put("/api/admin/articles")
        .json(&article)
        .unwrap()
        .send()
        .await
}
