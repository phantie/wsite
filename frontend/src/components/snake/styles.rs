#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;

pub fn btn_style() -> stylist::StyleSource {
    css! {"
        cursor: pointer;
        text-align: center;
        user-select: none;
        :hover {
            opacity: 0.8;
        }"
    }
}

pub fn big_btn_style() -> stylist::StyleSource {
    css! {"
        border-width: 4px;
        border-style: solid;
        padding: 30px 60px;
        font-size: 50px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: 0.3s;
    "}
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
