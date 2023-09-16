#![allow(unused_variables)]
use crate::components::imports::*;

pub struct Video;

pub enum VideoMsg {}

impl Component for Video {
    type Message = VideoMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <video
                controls=true
                src="/api/video"
                width="620">
                { "Unsupported video" }
            </video>
        }
        // html! {
        //     <video
        //         controls=true
        //         src="https://archive.org/download/BigBuckBunny_124/Content/big_buck_bunny_720p_surround.mp4"
        //         poster="https://peach.blender.org/wp-content/uploads/title_anouncement.jpg?x11217"
        //         width="620">
        //         { "Unsupported video" }
        //     </video>
        // }
    }
}
