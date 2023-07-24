#![allow(non_upper_case_globals)]

use crate::components::imports::*;

#[derive(Properties, PartialEq)]
pub struct ListProps {
    pub with: AttrValue,
    #[prop_or_default]
    pub children: Children,
}

pub struct Colored {
    pub style: stylist::StyleSource,
}

impl Component for Colored {
    type Message = ();
    type Properties = ListProps;

    fn create(ctx: &Context<Self>) -> Self {
        let color = ctx.props().with.clone();
        let style = css!(
            "
                display: inline;
                color: ${color};
            ",
            color = color
        );

        Self { style }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={ self.style.clone() }>
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
