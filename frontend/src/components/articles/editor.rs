#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::MarkdownPreview;

type Article = interfacing::ArticleWithId;

#[derive(PartialEq, Clone)]
pub enum ArticleEditorMode {
    Create,
    Edit(Article),
}

pub struct ArticleEditor {
    theme_ctx: ThemeCtxSub,
    refs: Refs,
    mode: ArticleEditorMode,
    // TODO
    article_history: Vec<Article>,
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
    NewArticleVersion(Article),
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
            ArticleEditorMode::Create => Article {
                // TODO this should be type without ID, and handled everywhere properly
                // it works fine, because server ignores this value
                id: "".into(),
                title: "".into(),
                public_id: "".into(),
                markdown: "".into(),
                draft: true,
            },
            ArticleEditorMode::Edit(article) => article.clone(),
        };

        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            refs: Refs::default(),
            mode: ctx.props().mode.clone(),
            article_history: vec![initial_article.clone()],
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

        let actions_block = match &self.mode {
            ArticleEditorMode::Create => {
                let onclick = {
                    let new_article = self.current_article_state.clone();
                    let navigator = ctx.link().navigator().unwrap();

                    ctx.link().callback_future(move |_| {
                        let new_article = new_article.clone();
                        let navigator = navigator.clone();

                        async move {
                            console::log!(format!("submitting: {:?}", new_article));
                            let r = request_article_post(&new_article).await.unwrap();
                            r.log_status();

                            let window = web_sys::window().unwrap();
                            match r.status() {
                                200 => {
                                    let navigator = navigator.clone();
                                    window.alert_with_message("Created!").unwrap();
                                    navigator.push(&Route::EditArticle {
                                        public_id: new_article.public_id,
                                    });
                                }
                                _ => {
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
            ArticleEditorMode::Edit(_article) => {
                let onclick = {
                    let new_article = self.current_article_state.clone();

                    ctx.link().callback_future(move |_| {
                        let new_article = new_article.clone();

                        async move {
                            console::log!(format!("submitting: {:?}", new_article));
                            let r = request_article_edit(&new_article).await.unwrap();
                            r.log_status();

                            let window = web_sys::window().unwrap();
                            match r.status() {
                                200 => {
                                    window.alert_with_message("Updated!").unwrap();
                                    Msg::NewArticleVersion(new_article)
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
                <PageTitle title={format!("New: {}", self.current_article_state.title)}/>
            },
            ArticleEditorMode::Edit(_) => html! {
                <PageTitle title={format!("Edit: {}", self.current_article_state.title)}/>
            },
        };

        html! {
            <DefaultStyling>
                { title }

                <div class={css!("display:flex;")}>
                    <div class={css!("height: 100vh; width: 100%;")}>
                        <MarkdownPreview {oninput} md={self.current_article_state.markdown.clone()}/>
                    </div>

                    <div class={metadata_classes}>
                        <div class={css!{"height: 80px;"}}/>

                        <h2><div>{ mode_display }</div></h2>

                        <div class={metadatum_classes.clone()}>
                            <label for="title_input">{ "Title" }</label>
                            <input oninput={title_oninput} name="title_input"
                                ref={self.refs.title_ref.clone()}
                                value={ self.current_article_state.title.clone() }
                            />
                        </div>
                        <div class={metadatum_classes.clone()}>
                            <label for="public_id_input">{ "Public ID" }</label>
                            <input oninput={public_id_oninput} name="public_id_input"
                                ref={self.refs.public_id_ref.clone()}
                                value={ self.current_article_state.public_id.clone() }
                            />
                        </div>

                        <div class={checkbox_classes.clone()}>
                            <label for="draft_input">{ "Draft" }</label>
                            <input oninput={draft_oninput} name="draft_input" type="checkbox"
                                ref={self.refs.draft_ref.clone()}
                                checked={ self.current_article_state.draft }
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
                self.current_article_state.title = value;
                true
            }
            Self::Message::PublicIDChanged(value) => {
                console::log!(format!("public ID changed from ArticleEditor"));
                self.current_article_state.public_id = value;
                true
            }
            Self::Message::MarkdownChanged(value) => {
                console::log!(format!("markdown changed from ArticleEditor"));
                self.current_article_state.markdown = value.to_string();
                true
            }
            Self::Message::DraftStateChanged(value) => {
                console::log!(format!("draft state changed from ArticleEditor"));
                self.current_article_state.draft = value;
                true
            }
            Self::Message::NewArticleVersion(value) => {
                console::log!(format!("new article version saved from ArticleEditor"));
                self.article_history.push(value);
                false
            }
            Self::Message::Nothing => false,
        }
    }
}

async fn request_article_post(article: &Article) -> request::SendResult {
    Request::static_post(routes().api.admin.articles)
        .json(&article)
        .unwrap()
        .send()
        .await
}

async fn request_article_edit(article: &Article) -> request::SendResult {
    Request::put("/api/admin/articles")
        .json(&article)
        .unwrap()
        .send()
        .await
}
