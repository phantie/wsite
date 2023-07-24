#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::Markdown;

pub struct MarkdownPreview {
    input_value: AttrValue,
    input_node_ref: NodeRef,
    theme_ctx: ThemeCtxSub,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or(Callback::noop())]
    pub oninput: Callback<AttrValue>,
    #[prop_or("".into())]
    pub md: AttrValue,
}

pub enum Msg {
    InputChanged(AttrValue),
    ThemeContextUpdate(ThemeCtx),
}

impl Component for MarkdownPreview {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            input_value: ctx.props().md.clone(),
            input_node_ref: NodeRef::default(),
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let text_color = &theme.text_color;
        let box_border_color = &theme.box_border_color;

        let md_preview_style = css!(
            "
            display: flex;
            height: 100%;
        "
        );
        let md_preview_classes = classes!(md_preview_style);

        let input_style = css!(
            "
            font-size: 150%;
            border: none;
            resize: none;
            height: 100%;
            width: 50%;
            padding: 1.5em;
            box-sizing: border-box;
            color: ${text_color};
            background-color: ${bg_color};
            border-right: 0.1em solid ${box_border_color};

            :focus {
                outline: none;
            }
        ",
            bg_color = bg_color,
            text_color = text_color,
            box_border_color = box_border_color
        );
        let input_classes = classes!(input_style);

        let preview_style = css!(
            "
            .markdown-body {
                font-size: 130%;
            }

            height: 100%;
            width: 50%;
            padding: 2em;
            box-sizing: border-box;
            overflow-x: auto;
            background-color: ${bg_color};
        ",
            bg_color = bg_color
        );
        let input_node_ref = self.input_node_ref.clone();

        let oninput = {
            let input_node_ref = self.input_node_ref.clone();
            ctx.link().callback(move |_| {
                let input_field = input_node_ref.cast::<HtmlInputElement>().unwrap();
                let value = input_field.value();
                Self::Message::InputChanged(value.into())
            })
        };

        html! {
            <>
                <div class={ md_preview_classes }>
                    <textarea { oninput }
                        ref={ input_node_ref }
                        class={ input_classes }
                        value= { self.input_value.clone() }
                    />
                    <div class={ preview_style }>
                        <Markdown md={ self.input_value.clone() }/>
                    </div>
                </div>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::InputChanged(value) => {
                self.input_value = value.clone();
                ctx.props().oninput.emit(value);
                true
            }
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Markdown Preview");
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }
}
