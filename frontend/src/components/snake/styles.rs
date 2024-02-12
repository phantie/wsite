#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;

pub fn base_btn_style() -> stylist::StyleSource {
    css! {"
        cursor: pointer;
        text-align: center;
        user-select: none;
        :hover {
            opacity: 0.8;
        }"
    }
}

pub fn btn_style() -> yew::Classes {
    yew::classes!(
        base_btn_style(),
        css! {"
        border-width: 4px;
        border-style: solid;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: 0.3s;
    "}
    )
}

pub fn big_btn_style() -> yew::Classes {
    yew::classes!(
        btn_style(),
        css! {"
        padding: 30px 60px;
        font-size: 50px;
    "}
    )
}

pub fn average_btn_style() -> yew::Classes {
    yew::classes!(
        btn_style(),
        css! {"
        padding: 15px 30px;
        font-size: 25px;
    "}
    )
}

pub fn centered_column_items() -> stylist::StyleSource {
    css! {"
        display: flex; justify-content: center;
        align-items: center; flex-direction: column;"
    }
}

pub fn input_style() -> stylist::StyleSource {
    css! {"
        width: 300px;
        font-size: 24px;
        padding: 20px;
        text-align:center;
    "}
}
