#![allow(unused)]

use crate::components::imports::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

use super::domain;

#[derive(Default, Clone)]
pub struct Refs {
    canvas_ref: NodeRef,
}

pub struct Snake {
    refs: Refs,

    snake: domain::Snake,
}

pub enum SnakeMsg {
    Advance,
    Nothing,
}

impl Component for Snake {
    type Message = SnakeMsg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            refs: Default::default(),
            snake: Default::default(),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let window = web_sys::window().unwrap();

        let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
        let height = window.inner_height().unwrap().as_f64().unwrap() as u32;

        let width = width - 15;
        let height = height - 5;

        let restart_button_onclick = {
            let canvas_ref = self.refs.canvas_ref.clone();
            ctx.link().callback(move |e| {
                let canvas = canvas_ref.clone().cast::<HtmlCanvasElement>().unwrap();
                console::log!(&canvas);
                console::log!("restart");
                Self::Message::Nothing
            })
        };

        #[allow(non_upper_case_globals)]
        let button_style = css! {"
            position: absolute;
            right: 100px;
            top: 10px;
        "};

        html! {
            <>
                <button class={ button_style } onclick={restart_button_onclick}>{ "Restart" }</button>
                <canvas ref={self.refs.canvas_ref.clone()} width={ width.to_string() } height={ height.to_string() }></canvas>
            </>
        }
    }

    #[allow(unused)]
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas_el = self
            .refs
            .canvas_ref
            .clone()
            .cast::<HtmlCanvasElement>()
            .unwrap();
        // console::log!(&canvas);

        let canvas_rendering_ctx_object = canvas_el.get_context("2d").unwrap().unwrap();

        let canvas_rendering_ctx =
            canvas_rendering_ctx_object.unchecked_into::<CanvasRenderingContext2d>();
        console::log!(&canvas_rendering_ctx);

        let r = canvas_rendering_ctx;

        self.draw_snake(&r);
        // r.move_to(0f64, 0f64);
        // r.line_to(200f64, 100f64);
        // r.stroke();
    }

    #[allow(unused)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Nothing => false,
            Self::Message::Advance => {
                self.snake.advance();
                true
            }
        }
    }
}

impl Snake {
    fn draw_snake(&self, r: &CanvasRenderingContext2d) {
        for section in &self.snake.sections {
            let domain::Pos { x, y } = section.start;
            r.move_to(x as f64, y as f64);

            let domain::Pos { x, y } = section.end;
            r.line_to(x as f64, y as f64);
        }
        r.stroke();

        let section = self.snake.sections.last().unwrap();
        let domain::Pos { x, y } = section.end;
        r.begin_path();
        r.arc(x as f64, y as f64, 20 as f64, 0 as f64, 2.0 * 3.14)
            .unwrap();
        r.stroke();
    }
}
