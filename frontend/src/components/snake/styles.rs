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
