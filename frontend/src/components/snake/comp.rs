#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_timers::callback::Interval;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use yew::html::Scope;

use super::domain;

const PAUSED: bool = false;

const PX_SCALE: f64 = 100.0;

const ADJUST_ALGO: AdjustAlgoChoice = AdjustAlgoChoice::None;

enum AdjustAlgoChoice {
    None,
    // redundant at least when "camera" is centered to the mouth
    For4thQuadrant,
}

#[derive(Clone, Copy)]
enum AdjustAlgo {
    None,
    For4thQuadrant { initial_adjustment: domain::Pos },
}

impl AdjustAlgo {
    fn apply(&self, pos: domain::Pos) -> domain::Pos {
        match self {
            Self::None => pos,
            Self::For4thQuadrant { initial_adjustment } => pos + initial_adjustment.clone(),
        }
    }
}

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

    fn set_canvas_size(&self, dims: Dimensions) {
        let canvas = self.canvas_el();
        canvas.set_height(dims.height);
        canvas.set_width(dims.width);
        console::log!("canvas resized to:", format!("{:?}", dims));
    }

    fn fit_canvas_to_window_size(&self) {
        let wd = window_dimensions();
        self.set_canvas_size(wd);
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
    fn snake(adjust_algo: AdjustAlgoChoice) -> domain::Snake {
        let initial_pos = match adjust_algo {
            AdjustAlgoChoice::None => domain::Pos::new(1, 1),
            // test with negative coords
            AdjustAlgoChoice::For4thQuadrant => domain::Pos::new(-2, 2),
        };

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

impl Domain {
    fn default(adjust_algo: AdjustAlgoChoice) -> Self {
        Self {
            snake: DomainDefaults::snake(adjust_algo),
            foods: DomainDefaults::foods(),
        }
    }
}

pub struct Snake {
    domain: Domain,
    adjust_algo: AdjustAlgo,

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
            _handle: Interval::new(millis, move || {
                link.send_message(if PAUSED {
                    SnakeMsg::Nothing
                } else {
                    SnakeMsg::Advance
                })
            }),
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

        let domain = Domain::default(ADJUST_ALGO);

        let adjust_algo = match ADJUST_ALGO {
            AdjustAlgoChoice::None => AdjustAlgo::None,
            AdjustAlgoChoice::For4thQuadrant => {
                let initial_adjustment = {
                    let snake_boundaries = domain.snake.boundaries();
                    let foods_boundaries = domain.foods.boundaries();
                    let total_boundaries = snake_boundaries.join_option(foods_boundaries);

                    fn adjust_for_negative(coord: i32) -> i32 {
                        if coord < 0 {
                            -coord
                        } else {
                            0
                        }
                    }

                    let x_adjust_for_negative = adjust_for_negative(total_boundaries.min.x);
                    let y_adjust_for_negative = adjust_for_negative(total_boundaries.min.y);

                    domain::Pos {
                        x: x_adjust_for_negative,
                        y: y_adjust_for_negative,
                    }
                };

                AdjustAlgo::For4thQuadrant { initial_adjustment }
            }
        };

        Self {
            domain,

            advance_interval: SnakeAdvanceInterval::default(ctx.link().clone()),

            adjust_algo,
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

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas_el = self.refs.canvas_el();

        if first_render {
            self.refs.fit_canvas_to_window_size();
        }

        let canvas_rendering_ctx_object = canvas_el.get_context("2d").unwrap().unwrap();

        let canvas_rendering_ctx =
            canvas_rendering_ctx_object.unchecked_into::<CanvasRenderingContext2d>();

        let r = canvas_rendering_ctx;

        r.set_stroke_style(&JsValue::from_str("white"));
        r.set_line_join("round");
        r.set_line_width(10f64);
        r.set_fill_style(&JsValue::from_str("black"));
        let wd = window_dimensions();
        r.fill_rect(0f64, 0f64, f64::from(wd.width), f64::from(wd.height));

        self.draw_snake(&r);
        self.draw_foods(&r);

        console::log!(format!("Snake: {:?}", self.domain.snake.boundaries()));
        console::log!(format!("Foods: {:?}", self.domain.foods.boundaries()));
        console::log!(format!(
            "Joined: {:?}",
            self.domain
                .snake
                .boundaries()
                .join_option(self.domain.foods.boundaries())
        ));
    }

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
                self.refs.fit_canvas_to_window_size();
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
                        if self.out_of_window_bounds(window_dimensions()) {
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
                self.domain = Domain::default(ADJUST_ALGO);
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
        let pos = TransformedPos::from(self.adjust_algo.apply(pos)) * PX_SCALE;

        // center camera to the mouth
        //
        // position of the mouth after the same transformations as of 'pos'
        let adjusted_mouth =
            TransformedPos::from(self.adjust_algo.apply(self.domain.snake.mouth())) * PX_SCALE;
        // target position - center of the window
        let wd = window_dimensions() / 2.0;
        let center_x = wd.width - adjusted_mouth.x;
        let center_y = wd.height - adjusted_mouth.y;

        TransformedPos::new(pos.x + center_x, pos.y + center_y)
    }

    pub fn out_of_window_bounds(&self, wd: Dimensions) -> bool {
        let mouth = self.transform_pos(self.domain.snake.mouth());
        mouth.x < 0f64
            || mouth.y < 0f64
            || mouth.x > f64::from(wd.width)
            || mouth.y > f64::from(wd.height)
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

fn window_dimensions() -> Dimensions {
    Dimensions::from(get_window())
}

fn get_document() -> Document {
    let window = get_window();
    window.document().unwrap()
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct FracDimensions {
    pub width: f64,
    pub height: f64,
}

impl std::ops::Div<f64> for Dimensions {
    type Output = FracDimensions;
    fn div(self, rhs: f64) -> Self::Output {
        Self::Output {
            width: f64::from(self.width) / rhs,
            height: f64::from(self.height) / rhs,
        }
    }
}

impl From<web_sys::Window> for Dimensions {
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

impl TransformedPos {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for TransformedPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for TransformedPos {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl From<domain::Pos> for TransformedPos {
    fn from(value: domain::Pos) -> Self {
        Self::new(f64::from(value.x), f64::from(value.y))
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
