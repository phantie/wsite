#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_timers::callback::Interval;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use yew::html::Scope;

use super::common::WindowSize;
use super::domain;

#[derive(Default, Clone)]
pub struct Refs {
    canvas_ref: NodeRef,
}

pub struct Listeners {
    kb_listener: EventListener,
    window_load_listener: EventListener,
    window_resize_listener: EventListener,
}

pub struct Snake {
    snake: domain::Snake,
    foods: domain::Foods,

    advance_interval: SnakeAdvanceInterval,

    refs: Refs,
    listeners: Listeners,
}

// in milliseconds
const SNAKE_ADVANCE_INTERVAL: u32 = 750;

struct SnakeAdvanceInterval {
    _handle: Interval,
}

impl SnakeAdvanceInterval {
    fn default(link: Scope<Snake>) -> Self {
        Self::init(SNAKE_ADVANCE_INTERVAL, link)
    }

    fn init(millis: u32, link: Scope<Snake>) -> Self {
        Self {
            _handle: Interval::new(millis, move || link.send_message(SnakeMsg::Advance)),
        }
    }
}

pub enum SnakeMsg {
    Advance,
    Restart,
    DirectionChange(domain::Direction),
    WindowLoaded,
    WindowResized,
    Nothing,
}

impl Component for Snake {
    type Message = SnakeMsg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let listeners = {
            let link = ctx.link().clone();
            let window_load_listener = EventListener::new(&get_window(), "load", move |event| {
                console::log!("event: window load");
                link.send_message(Self::Message::WindowLoaded);
            });

            let link = ctx.link().clone();
            let window_resize_listener =
                EventListener::new(&get_window(), "resize", move |event| {
                    console::log!("event: window resize");
                    link.send_message(Self::Message::WindowResized);
                });

            let link = ctx.link().clone();
            let kb_listener = EventListener::new_with_options(
                &get_document(),
                "keydown",
                EventListenerOptions::enable_prevent_default(),
                move |event| {
                    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();

                    enum KeyBoardEvent {
                        DirectionChange(domain::Direction),
                        None,
                    }
                    use KeyBoardEvent::*;

                    let kb_event = match event.key().as_str() {
                        "ArrowUp" => DirectionChange(domain::Direction::Up),
                        "ArrowDown" => DirectionChange(domain::Direction::Bottom),
                        "ArrowLeft" => DirectionChange(domain::Direction::Left),
                        "ArrowRight" => DirectionChange(domain::Direction::Right),
                        _ => None,
                    };

                    match kb_event {
                        DirectionChange(direction) => {
                            link.send_message(Self::Message::DirectionChange(direction))
                        }
                        None => {}
                    };
                },
            );

            Listeners {
                kb_listener,
                window_load_listener,
                window_resize_listener,
            }
        };

        Self {
            snake: Default::default(),
            foods: Default::default(),

            advance_interval: SnakeAdvanceInterval::default(ctx.link().clone()),

            refs: Default::default(),
            listeners,
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let restart_button_onclick = ctx.link().callback(move |e| Self::Message::Restart);

        let direction_onlick = |d: domain::Direction| {
            ctx.link()
                .callback(move |e| Self::Message::DirectionChange(d))
        };

        let button_style = css! {"
            border: 2px solid white;
            width: 50px; height: 20px;
            color: white;
            cursor: pointer;
            display: inline-block;
            padding: 7px 5px; margin: 2px 1px;
            text-align: center;
            transition: 0.3s;
            user-select: none;

            :hover {
                opacity: 0.8;
            }

            :active {
                transition: 0s;
                border-color: green;
                background-color: green;
            }
        "};

        let restart_button_style = css! {"
            position: absolute;
            right: 200px;
            top: 10px;   
        "};

        let move_button = |text: &str, d: domain::Direction| {
            html! {
                <div class={ button_style.clone() } onclick={direction_onlick(d)}>{ text }</div>
            }
        };

        html! {
            <>
                <div class={css!("display: flex; margin-top: 20px; position: absolute; left: 0; right: 0; align-items: center; flex-direction: column;")}>
                    <div>
                        { move_button("▲", domain::Direction::Up) }
                    </div>

                    <div>
                        { move_button("◄", domain::Direction::Left) }
                        { move_button("▼", domain::Direction::Bottom) }
                        { move_button("►", domain::Direction::Right) }
                    </div>
                </div>

                <div class={ vec![restart_button_style, button_style] } onclick={restart_button_onclick}>{ "Restart" }</div>
                <canvas
                    class={css!("position: absolute; z-index: -1;")}
                    ref={self.refs.canvas_ref.clone()}></canvas>
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

        let ws = WindowSize::from(get_window());

        let canvas_rendering_ctx_object = canvas_el.get_context("2d").unwrap().unwrap();

        let canvas_rendering_ctx =
            canvas_rendering_ctx_object.unchecked_into::<CanvasRenderingContext2d>();

        let r = canvas_rendering_ctx;

        r.set_stroke_style(&JsValue::from_str("white"));
        r.set_line_join("round");
        r.set_line_width(10f64);
        r.set_fill_style(&JsValue::from_str("black"));
        r.fill_rect(0f64, 0f64, ws.width as f64, ws.height as f64);

        self.draw_snake(&r);
        self.draw_foods(&r);
    }

    #[allow(unused)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Nothing => false,
            Self::Message::WindowResized => {
                ctx.link().send_message(Self::Message::WindowLoaded);
                false
            }
            Self::Message::WindowLoaded => {
                let canvas_el = self
                    .refs
                    .canvas_ref
                    .clone()
                    .cast::<HtmlCanvasElement>()
                    .unwrap();

                let window = get_window();
                let ws = WindowSize::from(window);

                canvas_el.set_height(ws.height as u32);
                canvas_el.set_width(ws.width as u32);

                console::log!("set new window size:", ws.height, ws.width);

                true
            }
            Self::Message::Advance => {
                let window = get_window();
                match self
                    .snake
                    .advance(WindowSize::from(window.clone()), &mut self.foods)
                {
                    domain::AdvanceResult::Success => {}
                    domain::AdvanceResult::OutOfBounds | domain::AdvanceResult::BitYaSelf => {
                        window.alert_with_message("game over");
                        // when game ends - auto restart
                        ctx.link().send_message(Self::Message::Restart);
                    }
                }
                true
            }
            Self::Message::Restart => {
                // drop old by replacement
                self.advance_interval = SnakeAdvanceInterval::default(ctx.link().clone());
                // place snake
                self.snake = Default::default();
                // replenish foods
                self.foods = Default::default();
                true
            }
            Self::Message::DirectionChange(direction) => {
                if self.snake.set_direction(direction).is_err() {
                    console::log!("cannot move into the opposite direction")
                }

                false
            }
        }
    }
}

impl Snake {
    fn draw_snake(&self, r: &CanvasRenderingContext2d) {
        r.begin_path();
        let domain::Pos { x, y } = self.snake.iter_vertices().next().unwrap();
        r.move_to(x as f64, y as f64);
        for domain::Pos { x, y } in self.snake.iter_vertices().skip(1) {
            r.line_to(x as f64, y as f64);
        }
        r.stroke();
        r.close_path();

        let domain::Pos { x, y } = self.snake.mouth();

        r.begin_path();
        r.arc(x as f64, y as f64, 20 as f64, 0 as f64, 2.0 * 3.14)
            .unwrap();
        r.set_fill_style(&JsValue::from_str("white"));
        r.fill();
        r.stroke();
        r.close_path();
    }

    fn draw_foods(&self, r: &CanvasRenderingContext2d) {
        for food in self.foods.as_ref() {
            let domain::Pos { x, y } = food.pos;
            r.begin_path();
            r.arc(x as f64, y as f64, 30 as f64, 0 as f64, 2.0 * 3.14)
                .unwrap();
            r.set_fill_style(&JsValue::from_str("white"));
            r.fill();
            r.stroke();
            r.close_path();
        }
    }
}

fn get_window() -> Window {
    web_sys::window().unwrap()
}

fn get_document() -> Document {
    let window = get_window();
    window.document().unwrap()
}
