use crate::components::imports::*;
use crate::components::MarkdownPreview;
#[allow(unused_imports)]
use crate::components::{ThemeCtx, ThemeCtxSub, Themes};

pub struct ArticleEditor {
    theme_ctx: ThemeCtxSub,
    refs: Refs,
    md_value: AttrValue,
}

#[derive(Default, Clone)]
pub struct Refs {
    title_ref: NodeRef,
    public_id_ref: NodeRef,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
    MarkdownChanged(AttrValue),
    Nothing,
}

impl Component for ArticleEditor {
    type Message = Msg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
            refs: Refs::default(),
            md_value: "".into(),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let contrast_bg_color = &theme.contrast_bg_color;
        let text_color = &theme.text_color;
        let box_border_color = &theme.box_border_color;

        let global_style = css!(
            "
                body {
                    background-color: ${bg_color};
                }
            ",
            bg_color = bg_color,
        );

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
        ",
            bg_color = bg_color,
        );
        let metadata_classes = metadata_style;

        let metadatum_style = css!(
            "
            width: 85%;
            margin-bottom: 10px;

            label {
                color: ${label_text_color};
                font-size: 1.5em;
                font-weight: bold;
                margin-bottom: 10px;
                display: block;
            }

            input {
                width: inherit;
                height: 40px;
                color: ${input_text_color};
                background-color: transparent;
                border: 3px solid ${box_border_color};
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
            label_text_color = text_color,
            input_text_color = text_color,
            box_border_color = box_border_color
        );

        let metadatum_classes = metadatum_style;

        let oninput = ctx.link().callback(Self::Message::MarkdownChanged);

        let onclick_save = {
            let title_ref = self.refs.title_ref.clone();
            let public_id_ref = self.refs.public_id_ref.clone();
            let md_value = self.md_value.to_string();

            ctx.link().callback_future(move |_| {
                let title_field = title_ref.cast::<HtmlInputElement>().unwrap();
                let public_id_field = public_id_ref.cast::<HtmlInputElement>().unwrap();

                let new_article = interfacing::Article {
                    public_id: public_id_field.value(),
                    title: title_field.value(),
                    markdown: md_value.clone(),
                };

                async move {
                    console::log!(format!("submitting: {:?}", new_article));
                    let r = request_article_post(&new_article).await.unwrap();
                    r.log_status();

                    let window = web_sys::window().unwrap();
                    match r.status() {
                        200 => {
                            window.alert_with_message("Created!").unwrap();
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
            <>
                <Global css={global_style}/>

                <div class={css!("display:flex;")}>
                    <div class={css!("height: 100vh; width: 100%;")}>
                        <MarkdownPreview {oninput}/>
                    </div>

                    <div class={metadata_classes}>
                        <div class={css!{"height: 80px;"}}/>

                        <div class={metadatum_classes.clone()}>
                            <label for="title_input">{ "Title" }</label>
                            <input name="title_input"
                                ref={self.refs.title_ref.clone()}/>
                        </div>
                        <div class={metadatum_classes.clone()}>
                            <label for="public_id_input">{ "Public ID" }</label>
                            <input name="public_id_input"
                                ref={self.refs.public_id_ref.clone()}/>
                        </div>
                        <div onclick={onclick_save} class={action_classes.clone()}>{ "Save" }</div>
                    </div>
                </div>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Markdown Preview");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::MarkdownChanged(value) => {
                console::log!(format!("markdown changed from ArticleEditor"));
                self.md_value = value;
                true
            }
            _ => false,
        }
    }
}

async fn request_article_post(article: &interfacing::Article) -> request::SendResult {
    Request::static_post(routes().api.admin.articles)
        .json(&article)
        .unwrap()
        .send()
        .await
}
