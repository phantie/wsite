#![allow(non_upper_case_globals)]

use crate::components::imports::*;
use crate::components::MarkdownPreview;

pub struct MarkdownPreviewPage;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or("".into())]
    pub md: AttrValue,
}

impl Component for MarkdownPreviewPage {
    type Message = ();
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let wrapper_classes = css!(
            "
                height: 100vh;
            "
        );

        html! {
            <>
                <PageTitle title={"Markdown preview"}/>
                <div class={ wrapper_classes }>
                    <MarkdownPreview md={ctx.props().md.clone()}/>
                </div>
            </>
        }
    }
}
