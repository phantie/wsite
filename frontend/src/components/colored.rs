use crate::components::imports::*;

#[derive(Properties, PartialEq)]
pub struct ListProps {
    pub with: AttrValue,
    #[prop_or_default]
    pub children: Children,
}

pub struct Colored {
    pub style: Style,
}

impl Colored {
    fn style(color: AttrValue) -> Style {
        style!(
            "
                display: inline;
                color: ${color};
            ",
            color = color
        )
        .unwrap()
    }
}

impl Component for Colored {
    type Message = ();
    type Properties = ListProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            style: Self::style(ctx.props().with.clone()),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={self.style.clone()}>
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
