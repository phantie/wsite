#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_timers::callback::Interval;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use yew::html::Scope;

use super::domain;

pub const PX_SCALE: f64 = 100.0;

#[derive(Default, Clone)]
pub struct CtrlBtnRefs {
    up_btn_ref: NodeRef,
    down_btn_ref: NodeRef,
    left_btn_ref: NodeRef,
    right_btn_ref: NodeRef,
}

impl CtrlBtnRefs {
    fn from_direction(&self, direction: domain::Direction) -> NodeRef {
        match direction {
            domain::Direction::Up => self.up_btn_ref.clone(),
            domain::Direction::Bottom => self.down_btn_ref.clone(),
            domain::Direction::Left => self.left_btn_ref.clone(),
            domain::Direction::Right => self.right_btn_ref.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Refs {
    canvas_ref: NodeRef,
    ctrl_brn_refs: CtrlBtnRefs,
}

impl Refs {
    fn canvas_el(&self) -> HtmlCanvasElement {
        self.canvas_ref.clone().cast::<HtmlCanvasElement>().unwrap()
    }

    fn set_canvas_size(&self, dims: Dimentions) {
        let canvas = self.canvas_el();
        canvas.set_height(dims.height);
        canvas.set_width(dims.width);
    }

    fn ctrl_btn_el(&self, direction: domain::Direction) -> HtmlElement {
        self.ctrl_brn_refs
            .from_direction(direction)
            .clone()
            .cast::<HtmlElement>()
            .unwrap()
    }
}

pub struct Listeners {
    kb_listener: EventListener,
    window_load_listener: EventListener,
    window_resize_listener: EventListener,
}

pub struct Domain {
    snake: domain::Snake,
    foods: domain::Foods,
}

struct DomainDefaults;

impl DomainDefaults {
    fn snake() -> domain::Snake {
        let initial_pos = domain::Pos::new(1, 1);

        let sections = domain::Sections::from_directions(
            initial_pos,
            [
                domain::Direction::Bottom,
                domain::Direction::Right,
                domain::Direction::Bottom,
            ],
        );
        assert!(sections.len() >= 2, "snake must have at least ... sections");

        // continue moving in the same direction
        let direction = sections.head().direction();

        domain::Snake {
            sections,
            direction,
        }
    }

    fn foods() -> domain::Foods {
        let values = vec![
            domain::Food::new(2, 5),
            domain::Food::new(3, 6),
            domain::Food::new(6, 3),
            domain::Food::new(7, 4),
        ];

        domain::Foods { values }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            snake: DomainDefaults::snake(),
            foods: DomainDefaults::foods(),
        }
    }
}

pub struct Snake {
    domain: Domain,

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
    FitCanvasToWindowSize,
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
            domain: Default::default(),

            advance_interval: SnakeAdvanceInterval::default(ctx.link().clone()),

            refs: Default::default(),
            listeners,
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let restart_button_onclick = ctx.link().callback(move |e| Self::Message::Restart);

        let direction_onlick = |direction| {
            ctx.link()
                .callback(move |_| Self::Message::DirectionChange(direction))
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

        "};

        let restart_button_style = css! {"
            position: absolute;
            right: 200px;
            top: 10px;   
        "};

        let move_button = |text: &str, direction| {
            html! {
                <div
                    ref={ self.refs.ctrl_brn_refs.from_direction(direction) }
                    class={ button_style.clone() }
                    onclick={ direction_onlick(direction) }>{ text }</div>
            }
        };

        html! {
            <>
                <Global css={".active_btn { transition: 0.2s; border-color: green; background-color: green; }"}/>
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
        let canvas_el = self.refs.canvas_el();

        let ws = Dimentions::from(get_window());

        if first_render {
            self.refs.set_canvas_size(ws);
        }

        let canvas_rendering_ctx_object = canvas_el.get_context("2d").unwrap().unwrap();

        let canvas_rendering_ctx =
            canvas_rendering_ctx_object.unchecked_into::<CanvasRenderingContext2d>();

        let r = canvas_rendering_ctx;

        r.set_stroke_style(&JsValue::from_str("white"));
        r.set_line_join("round");
        r.set_line_width(10f64);
        r.set_fill_style(&JsValue::from_str("black"));
        r.fill_rect(0f64, 0f64, f64::from(ws.width), f64::from(ws.height));

        self.draw_snake(&r);
        self.draw_foods(&r);
    }

    #[allow(unused)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Nothing => false,
            Self::Message::WindowLoaded => {
                ctx.link()
                    .send_message(Self::Message::FitCanvasToWindowSize);
                false
            }
            Self::Message::WindowResized => {
                ctx.link()
                    .send_message(Self::Message::FitCanvasToWindowSize);
                false
            }
            Self::Message::FitCanvasToWindowSize => {
                let ws = Dimentions::from(get_window());
                self.refs.set_canvas_size(ws);

                console::log!("resized canvas to:", ws.height, ws.width);

                true
            }
            Self::Message::Advance => {
                let game_over = || {
                    get_window().alert_with_message("game over");
                    // when game ends - auto restart
                    ctx.link().send_message(Self::Message::Restart);
                };

                match self.domain.snake.advance(&mut self.domain.foods) {
                    domain::AdvanceResult::Success => {
                        if self.out_of_window_bounds(Dimentions::from(get_window())) {
                            game_over();
                        }
                    }
                    domain::AdvanceResult::BitYaSelf => game_over(),
                }
                true
            }
            Self::Message::Restart => {
                // drop old by replacement
                self.advance_interval = SnakeAdvanceInterval::default(ctx.link().clone());
                // reset domain items
                self.domain = Domain::default();
                true
            }
            Self::Message::DirectionChange(direction) => {
                if self.domain.snake.set_direction(direction).is_err() {
                    console::log!("cannot move into the opposite direction")
                }

                let btn = self.refs.ctrl_btn_el(direction);

                if !btn.class_name().contains("active_btn") {
                    btn.set_class_name(&format!("active_btn {}", btn.class_name()));
                }

                gloo_timers::callback::Timeout::new(200, move || {
                    btn.set_class_name(&btn.class_name().replace("active_btn ", ""));
                })
                .forget();

                false
            }
        }
    }
}

impl Snake {
    pub fn transform_pos(&self, pos: domain::Pos) -> TransformedPos {
        let scale = PX_SCALE;

        let pos = TransformedPos::from(pos);
        TransformedPos {
            x: pos.x * scale,
            y: pos.y * scale,
        }
    }

    pub fn out_of_window_bounds(&self, ws: Dimentions) -> bool {
        let mouth = self.transform_pos(self.domain.snake.mouth());
        mouth.x < 0f64
            || mouth.y < 0f64
            || mouth.x > f64::from(ws.width)
            || mouth.y > f64::from(ws.height)
    }

    fn draw_snake(&self, r: &CanvasRenderingContext2d) {
        let TransformedPos { x, y } =
            self.transform_pos(self.domain.snake.iter_vertices().next().unwrap());
        r.begin_path();
        r.move_to(x, y);
        for TransformedPos { x, y } in self
            .domain
            .snake
            .iter_vertices()
            .skip(1)
            .map(|v| self.transform_pos(v))
        {
            r.line_to(x, y);
        }
        r.stroke();
        r.close_path();

        let TransformedPos { x, y } = self.transform_pos(self.domain.snake.mouth());
        r.begin_path();
        r.cirle(x, y, 20f64);
        r.set_fill_style(&JsValue::from_str("white"));
        r.fill();
        r.stroke();
        r.close_path();
    }

    fn draw_foods(&self, r: &CanvasRenderingContext2d) {
        for food in self.domain.foods.as_ref() {
            let TransformedPos { x, y } = self.transform_pos(food.pos);
            r.begin_path();
            r.cirle(x, y, 30f64);
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

#[derive(Clone, Copy)]
pub struct Dimentions {
    pub width: u32,
    pub height: u32,
}

impl From<web_sys::Window> for Dimentions {
    fn from(value: web_sys::Window) -> Self {
        let width = value.inner_width().unwrap().as_f64().unwrap() as u32;
        let height = value.inner_height().unwrap().as_f64().unwrap() as u32;
        Self { width, height }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TransformedPos {
    pub x: f64,
    pub y: f64,
}

impl From<domain::Pos> for TransformedPos {
    fn from(value: domain::Pos) -> Self {
        Self {
            x: f64::from(value.x),
            y: f64::from(value.y),
        }
    }
}

trait CanvasRenderingContext2dExtend {
    fn cirle(&self, x: f64, y: f64, radius: f64);
}

impl CanvasRenderingContext2dExtend for CanvasRenderingContext2d {
    fn cirle(&self, x: f64, y: f64, radius: f64) {
        self.arc(x, y, radius, 0f64, 2.0 * std::f64::consts::PI)
            .unwrap();
    }
}
