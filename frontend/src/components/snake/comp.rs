#![allow(unused, non_upper_case_globals)]
use std::collections::HashSet;

use crate::components::imports::*;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_timers::callback::Interval;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use yew::html::Scope;

use super::domain;

const PAUSED: bool = false;

// greater value - closer camera
const PX_SCALE: f64 = 70.0;

const ADJUST_ALGO: AdjustAlgoChoice = AdjustAlgoChoice::None;

const CAMERA: Camera = Camera::BoundariesCentered;

const MAP_BOUNDARIES_X: i32 = 10;
const MAP_BOUNDARIES_Y: i32 = 4;

pub enum Camera {
    MouthCentered,
    BoundariesCentered,
}

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
    boundaries: domain::Boundaries,
}

struct DomainDefaults;

impl DomainDefaults {
    fn snake(adjust_algo: AdjustAlgoChoice) -> domain::Snake {
        let rand_section_len = || rand_from_iterator(3..5);

        let initial_pos = match adjust_algo {
            AdjustAlgoChoice::None => domain::Pos::new(5, 5),
            // test with negative coords
            AdjustAlgoChoice::For4thQuadrant => domain::Pos::new(-2, 2),
        };

        let mut directions = vec![];
        for _ in 0..rand_section_len() {
            directions.push(rand_direction(
                directions.last().map(domain::Direction::opposite),
            ));
        }

        let sections = domain::Sections::from_directions(initial_pos, directions);
        assert!(sections.len() >= 2, "snake must have at least ... sections");

        // continue moving in the same direction
        let direction = sections.head().direction();

        domain::Snake {
            sections,
            direction,
        }
    }

    fn foods(
        food_count: i32,
        boundaries: domain::Boundaries,
        taken_positions: impl Iterator<Item = domain::Pos>,
    ) -> domain::Foods {
        let b = boundaries;
        let b = domain::Boundaries {
            min: domain::Pos::new(b.min.x + 1, b.min.y + 1),
            max: domain::Pos::new(b.max.x - 1, b.max.y - 1),
        };

        let mut positions = HashSet::new();
        for x in (b.min.x)..(b.max.x) {
            for y in (b.min.y)..(b.max.y) {
                positions.insert(domain::Pos::new(x, y));
            }
        }

        // let snake_vertices = snake.iter_vertices().collect::<HashSet<_>>();
        let taken_positions = taken_positions.collect::<HashSet<_>>();
        let mut vacant_food_positions = positions
            .difference(&taken_positions)
            .collect::<HashSet<_>>();

        let mut values = vec![];
        for _ in 0..food_count {
            if vacant_food_positions.len() > 0 {
                let food_pos = rand_from_iterator(vacant_food_positions.iter());
                values.push(**food_pos);
                let _removed = vacant_food_positions.remove(*food_pos);
            } else {
                break;
            }
        }

        let values = values.into_iter().map(domain::Food::from).collect();
        domain::Foods { values }
    }

    fn boundaries(snake: &domain::Snake) -> domain::Boundaries {
        snake
            .mouth()
            .boundaries_in_radius(MAP_BOUNDARIES_X, MAP_BOUNDARIES_Y)
    }
}

impl Domain {
    fn default(adjust_algo: AdjustAlgoChoice) -> Self {
        let snake = DomainDefaults::snake(adjust_algo);
        let boundaries = DomainDefaults::boundaries(&snake);
        Self {
            foods: DomainDefaults::foods(
                rand_from_iterator(10..15),
                boundaries,
                snake.iter_vertices(),
            ),
            boundaries,
            snake,
        }
    }
}

pub struct Snake {
    domain: Domain,

    advance_interval: SnakeAdvanceInterval,

    camera: Camera,
    adjust_algo: AdjustAlgo,

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
    CameraChange(Camera),
    CameraToggle,
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
                        Restart,
                        None,
                        CameraToggle,
                    }
                    use KeyBoardEvent::*;

                    let kb_event = match event.key().as_str() {
                        "ArrowUp" => DirectionChange(domain::Direction::Up),
                        "ArrowDown" => DirectionChange(domain::Direction::Bottom),
                        "ArrowLeft" => DirectionChange(domain::Direction::Left),
                        "ArrowRight" => DirectionChange(domain::Direction::Right),
                        "r" | "R" => Restart,
                        "c" | "C" => CameraToggle,
                        _ => None,
                    };

                    let message = match kb_event {
                        DirectionChange(direction) => Self::Message::DirectionChange(direction),
                        Restart => Self::Message::Restart,
                        None => Self::Message::Nothing,
                        CameraToggle => Self::Message::CameraToggle,
                    };
                    link.send_message(message)
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
            camera: CAMERA,

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
            width: 80px; height: 20px;
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

        let camera_button_style = css! {"
            position: absolute;
            right: 350px;
            top: 10px;
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

        let camera_button_onclick = ctx.link().callback(move |e| Self::Message::CameraToggle);

        html! {
            <>
                <Global css={".active_btn { transition: 0.2s; border-color: green; background-color: green; }"}/>
                <PageTitle title={"Snake"}/>
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

                <div class={ vec![camera_button_style, button_style.clone()] } onclick={camera_button_onclick}>{ "Camera (C)" }</div>
                <div class={ vec![restart_button_style, button_style] } onclick={restart_button_onclick}>{ "Restart (R)" }</div>
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
        r.set_line_width(px_scale(0.1));
        r.set_fill_style(&JsValue::from_str("black"));
        let wd = window_dimensions();
        r.fill_rect(0f64, 0f64, f64::from(wd.width), f64::from(wd.height));

        self.draw_snake(&r);
        self.draw_foods(&r);
        self.draw_boundaries(&r);

        // console::log!("random int (0..1)", rand_from_iterator(0..1));
        // console::log!("random int (0..2)", rand_from_iterator(0..2));
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
                        if self.out_of_bounds() {
                            game_over();
                        }

                        // when no food, replenish
                        if self.domain.foods.empty() {
                            self.domain.foods = DomainDefaults::foods(
                                rand_from_iterator(10..15),
                                self.domain.boundaries,
                                self.domain.snake.iter_vertices(),
                            );
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
            Self::Message::CameraChange(camera) => {
                self.camera = camera;
                true
            }
            Self::Message::CameraToggle => {
                let next_camera = match self.camera {
                    Camera::MouthCentered => Camera::BoundariesCentered,
                    Camera::BoundariesCentered => Camera::MouthCentered,
                };
                ctx.link()
                    .send_message(Self::Message::CameraChange(next_camera));
                false
            }
        }
    }
}

impl Snake {
    pub fn transform_pos(&self, pos: domain::Pos) -> TransformedPos {
        let pos = TransformedPos::from(self.adjust_algo.apply(pos)) * PX_SCALE;

        match self.camera {
            Camera::MouthCentered => {
                // center camera to the mouth
                //
                // position of the mouth after the same transformations as of 'pos'
                let adjusted_mouth =
                    TransformedPos::from(self.adjust_algo.apply(self.domain.snake.mouth()))
                        * PX_SCALE;
                // target position - center of the window
                let wd = window_dimensions() / 2.0;
                let to_center_x = wd.width - adjusted_mouth.x;
                let to_center_y = wd.height - adjusted_mouth.y;

                TransformedPos::new(pos.x + to_center_x, pos.y + to_center_y)
            }
            Camera::BoundariesCentered => {
                // center camera to the boundaries center
                //
                let b = self.domain.boundaries;
                let b_max = self.adjust_algo.apply(b.max);
                let b_min = self.adjust_algo.apply(b.min);
                let adjusted_boundaries_center = TransformedPos::new(
                    (b_min.x + b_max.x) as f64 / 2.0,
                    (b_min.y + b_max.y) as f64 / 2.0,
                ) * PX_SCALE;
                let wd = window_dimensions() / 2.0;
                let to_center_x = wd.width - adjusted_boundaries_center.x;
                let to_center_y = wd.height - adjusted_boundaries_center.y;

                TransformedPos::new(pos.x + to_center_x, pos.y + to_center_y)
            }
        }
    }

    pub fn out_of_bounds(&self) -> bool {
        let mouth = self.domain.snake.mouth();
        match self.domain.boundaries.relation(mouth) {
            domain::RelationToBoundaries::Inside => false,
            domain::RelationToBoundaries::Touching => true,
            domain::RelationToBoundaries::Outside => true,
        }
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
        r.cirle(x, y, px_scale(0.2));
        r.set_fill_style(&JsValue::from_str("white"));
        r.fill();
        r.stroke();
        r.close_path();
    }

    fn draw_foods(&self, r: &CanvasRenderingContext2d) {
        for food in self.domain.foods.as_ref() {
            let TransformedPos { x, y } = self.transform_pos(food.pos);
            r.begin_path();
            r.cirle(x, y, px_scale(0.3));
            r.set_fill_style(&JsValue::from_str("white"));
            r.fill();
            r.stroke();
            r.close_path();
        }
    }

    fn draw_boundaries(&self, r: &CanvasRenderingContext2d) {
        let TransformedPos { x, y } = self.transform_pos(self.domain.boundaries.left_top());

        r.begin_path();
        r.move_to(x, y);

        for pos in [
            self.domain.boundaries.right_top(),
            self.domain.boundaries.right_bottom(),
            self.domain.boundaries.left_bottom(),
            self.domain.boundaries.left_top(),
        ] {
            let TransformedPos { x, y } = self.transform_pos(pos);
            r.line_to(x, y);
        }
        r.stroke();
        r.close_path();
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

// use in drawing to preserve aspect ratio with differing camera distances
fn px_scale(value: f64) -> f64 {
    value * PX_SCALE
}

fn rand_from_iterator<Iter, I>(rng: Iter) -> <Iter as std::iter::Iterator>::Item
where
    Iter: IntoIterator<Item = I> + ExactSizeIterator,
{
    let random_float_from_zero_to_one = js_sys::Math::random();

    // optimized:
    // part = 1.0 / rng.len()
    // idx = random_float_from_zero_to_one / part
    let idx = (random_float_from_zero_to_one * rng.len() as f64) as usize;

    rng.enumerate()
        .find(|(i, v)| i == &idx)
        .map(|(_, v)| v)
        .expect("iterator not to be empty")
}

fn rand_direction(except: Option<domain::Direction>) -> domain::Direction {
    let mut directions = vec![
        domain::Direction::Up,
        domain::Direction::Bottom,
        domain::Direction::Left,
        domain::Direction::Right,
    ];

    match except {
        None => {}
        Some(except) => {
            let i = directions
                .iter()
                .enumerate()
                .find(|(i, d)| d == &&except)
                .map(|(i, _)| i)
                .unwrap();
            directions.remove(i);
        }
    }

    rand_from_iterator(directions.into_iter())
}
