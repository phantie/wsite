use crate::components::imports::*;

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub message: String,
}

#[styled_component]
pub fn ErrorMessage(props: &ErrorProps) -> Html {
    let message = &props.message;

    let css = css! {"
        color: rgb(248 83 20);
    "};

    html! {<h1 class={css}>{ message }</h1>}
}
