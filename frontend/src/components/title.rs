use crate::components::imports::*;

// drawbacks:
//      evaluation order matters, and it's top to down
//      so set title only from top components
// advantages:
//      simple, available from anywhere, efficient dynamic updating
pub struct PageTitle;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: AttrValue,
}

impl Component for PageTitle {
    type Message = ();
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        console::log!(format!("setting title: {:?}", &ctx.props().title));
        document.set_title(&ctx.props().title);
        html! {}
    }
}
