use crate::components::imports::*;
use crate::components::MarkdownPreview;

pub struct MarkdownPreviewPage;

impl Component for MarkdownPreviewPage {
    type Message = ();
    type Properties = ();

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
                    <MarkdownPreview/>
                </div>
            </>
        }
    }
}
