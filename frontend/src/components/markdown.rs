use crate::components::imports::*;

pub struct Markdown {
    theme_ctx: ThemeCtxSub,
}

pub enum Msg {
    ThemeContextUpdate(ThemeCtx),
}

#[derive(Properties, PartialEq)]

pub struct Props {
    pub md: AttrValue,
}

impl Component for Markdown {
    type Message = Msg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Markdown");
                self.theme_ctx.set(theme_ctx);
                true
            }
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let md_body = parse_md(ctx.props().md.as_str());

        let theme = self.theme_ctx.as_ref();

        let bg_color = &theme.bg_color;
        let code_bg_color = &theme.contrast_bg_color;
        let text_color = &theme.text_color;
        let link_color = &theme.link_color;

        let global_style = css!(
            "
                .markdown-body pre {
                    background-color: ${code_bg_color};
                }

                .markdown-body a:not(:has(code)):after {
                    content: \"ᴴ\";
                    margin-left: 0.1em;
                    margin-right: 0.1em;
                }

                .markdown-body a > code:last-of-type:after {
                    content: \"ᴴ\";
                    margin-left: 0.1em;
                    margin-right: 0.1em;
                }

                .markdown-body a > code {
                    background-color: ${code_bg_color};
                }

                .markdown-body a {
                    color: ${link_color};
                }

                .markdown-body img {
                    background-color: transparent;
                }
            ",
            code_bg_color = code_bg_color,
            link_color = link_color
        );

        let style = css!(
            "
                background-color: ${bg_color};
                color: ${text_color};
            ",
            bg_color = bg_color,
            text_color = text_color,
        );

        html! {
            <>
                <Global css={global_style}/>
                <div class={ classes!("markdown-body", style) }>
                    { md_body }
                </div>
            </>
        }
    }
}

pub fn parse_md(markdown_input: &str) -> yew::virtual_dom::VNode {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    yew::virtual_dom::VNode::from_html_unchecked(html_output.into())
}
