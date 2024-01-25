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
const STATE: State = State::NotBegun {
    inner: NotBegunState::ModeSelection,
};
// const STATE: State = State::Begun { paused: false };

const ADJUST_ALGO: AdjustAlgoChoice = AdjustAlgoChoice::None;

const CAMERA: Camera = Camera::BoundariesCentered;

const SNAKE_ADVANCE_INTERVAL: u32 = 450; // in milliseconds

const MAP_BOUNDARIES_X: i32 = 10;
const MAP_BOUNDARIES_Y: i32 = 10;

const PANEL_PX_WIDTH: u32 = 350;

const SNAKE_BODY_WIDTH: f64 = 0.9;
const FOOD_DIAMETER: f64 = 0.6;

pub struct Snake {
    domain: Domain,
    state: State,

    // true if state changed from NotBegun to Begun
    canvas_requires_fit: bool,

    advance_interval: SnakeAdvanceInterval,

    // greater value - closer camera
    px_scale: f64,
    camera: Camera,
    adjust_algo: AdjustAlgo,

    refs: Refs,
    listeners: Listeners,

    theme_ctx: ThemeCtxSub,
}

pub enum SnakeMsg {
    Advance,
    Restart,
    DirectionChange(domain::Direction),
    WindowLoaded,
    WindowResized,
    FitCanvasImmediately,
    CameraChange(Camera),
    CameraToggle,
    ThemeContextUpdate(ThemeCtx),
    Nothing,
    StateChange(State),
    Begin,
    PauseUnpause,
    ToMenu,
}

impl Component for Snake {
    type Message = SnakeMsg;
    type Properties = ();

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let domain = Domain::default();
        let px_scale = calc_px_scale(canvas_target_dimensions(), domain.boundaries);
        let adjust_algo = ADJUST_ALGO.into_algo(&domain);

        let state = STATE;

        let mut advance_interval = SnakeAdvanceInterval::create(ctx.link().clone());
        match state {
            State::Begun { .. } => advance_interval.start(),
            State::NotBegun { .. } => {}
        }

        Self {
            domain,
            state,

            canvas_requires_fit: match state {
                State::Begun { .. } => true,
                State::NotBegun { .. } => false,
            },

            advance_interval,

            px_scale,
            adjust_algo,
            camera: CAMERA,

            refs: Default::default(),
            listeners: Listeners::init(ctx.link().clone()),

            theme_ctx: ThemeCtxSub::subscribe(ctx, Self::Message::ThemeContextUpdate),
        }
    }

    #[allow(unused_variables)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        // TODO refactor this function
        // especially parts that match state

        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;
        let contrast_bg_color = &theme.contrast_bg_color;
        let text_color = &theme.text_color;

        let btn_style = css! {"
            border: 2px solid ${box_border_color};
            width: 80px; height: 20px;
            color: ${text_color};
            cursor: pointer;
            display: inline-block;
            padding: 7px 5px;
            text-align: center;
            user-select: none;
            :hover {
                opacity: 0.8;
            }
        ",
            box_border_color = box_border_color,
            text_color = text_color
        };

        let margin_top_btn_style = css! {"margin-top: 20px;"};
        let margin_bottom_btn_style = css! {"margin-bottom: 20px;"};

        let panel_style = css! {"
            width: ${width}px;
            background-color: ${bg_color};
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            outline: 5px groove ${box_border_color};
        ",
            width = PANEL_PX_WIDTH,
            box_border_color = box_border_color,
            bg_color = bg_color
        };

        let wrapper_style = css! {"display: flex;"};

        let global_style = css! {"
            .active_btn {
                transition: 0.2s;
                border-color: ${box_border_color};
                background-color: ${box_border_color};
                color: ${bg_color};
            }",
                box_border_color = box_border_color,
                bg_color = bg_color
        };

        let btns = {
            match self.state {
                State::Begun { .. } => {
                    let direction_btn = |text: &str, direction| {
                        let direction_btn_onlick = |direction| {
                            ctx.link()
                                .callback(move |_| Self::Message::DirectionChange(direction))
                        };

                        let direction_btn_style = css! {"margin: 2px 1px;"};

                        html! {
                            <div
                                ref={ self.refs.ctrl_brn_refs.from_direction(direction) }
                                class={ vec![btn_style.clone(), direction_btn_style] }
                                onclick={ direction_btn_onlick(direction) }>{ text }</div>
                        }
                    };

                    let camera_btn_style = margin_top_btn_style.clone();

                    let camera_btn_onclick =
                        ctx.link().callback(move |e| Self::Message::CameraToggle);

                    let restart_btn_onclick = ctx.link().callback(move |e| Self::Message::Restart);

                    let pause_btn_onclick =
                        ctx.link().callback(move |e| Self::Message::PauseUnpause);

                    let to_main_screen_btn_onclick =
                        ctx.link().callback(move |e| Self::Message::ToMenu);

                    html! {
                        <>
                            <div ref={self.refs.btn_refs.menu_btn_ref.clone()} class={ vec![margin_bottom_btn_style.clone(), btn_style.clone()] } onclick={to_main_screen_btn_onclick}>{ "Menu" }</div>

                            <div class={css!("display: flex; align-items: center; flex-direction: column;")}>
                                <div>
                                    { direction_btn("▲", domain::Direction::Up) }
                                </div>

                                <div>
                                    { direction_btn("◄", domain::Direction::Left) }
                                    { direction_btn("▼", domain::Direction::Bottom) }
                                    { direction_btn("►", domain::Direction::Right) }
                                </div>
                            </div>

                            <div ref={self.refs.btn_refs.camera_btn_ref.clone()} class={ vec![margin_top_btn_style.clone(), btn_style.clone()] } onclick={camera_btn_onclick}>{ "Camera (C)" }</div>
                            <div ref={self.refs.btn_refs.restart_btn_ref.clone()} class={ vec![margin_top_btn_style.clone(), btn_style.clone()] } onclick={restart_btn_onclick}>{ "Restart (R)" }</div>
                            <div ref={self.refs.btn_refs.pause_btn_ref.clone()} class={ vec![btn_style.clone(), margin_top_btn_style.clone()] } onclick={pause_btn_onclick}>{ "Pause (P)" }</div>
                        </>
                    }
                }
                State::NotBegun { .. } => {
                    let text_wrapper_style = css! {"
                        padding: 50px 35px;
                        color: ${text_color};
                        font-size: 25px;
                        font-family: 'Iosevka Web';
                    ",
                        text_color = text_color
                    };

                    html! {
                        <div class={text_wrapper_style}>
                            <h2>{"Snake"}</h2>
                            <p>{ "Goal - grow in size." }</p>
                            <p>{ "Use arrow keyboard buttons for controls." }</p>
                            <p>{ "Other buttons:" }</p>
                            <p><b>{ "R" }</b>{ " for Restart" }</p>
                            <p><b>{ "C" }</b>{ " for Camera" }</p>
                            <p><b>{ "P" }</b>{ " for Pause" }</p>
                        </div>
                    }
                }
            }
        };

        let main_area = match self.state {
            State::Begun { .. } => {
                html! { <canvas ref={self.refs.canvas_ref.clone() }></canvas> }
            }
            State::NotBegun { inner } if inner == NotBegunState::ModeSelection => {
                let sp_onclick = ctx.link().callback(move |e| {
                    Self::Message::StateChange(State::NotBegun {
                        inner: NotBegunState::Initial,
                    })
                });

                let mp_onclick = ctx.link().callback(move |e| {
                    Self::Message::StateChange(State::NotBegun {
                        inner: NotBegunState::MPCreateJoinLobby,
                    })
                });

                html! {
                    <>
                    <button onclick={ sp_onclick }>{ "Singleplayer" }</button>
                    <button onclick={ mp_onclick }>{ "Multiplayer" }</button>
                    </>
                }
            }
            State::NotBegun { inner } if inner == NotBegunState::MPCreateJoinLobby => {
                html! {
                    <>
                    <button>{ "Create server" }</button>
                    <button>{ "Join server" }</button>
                    </>
                }
            }
            State::NotBegun { inner } => {
                let canvas_overlay_style = css! {"
                    height: 100vh;
                    width: calc(100% - 350px);
                    background-color: ${bg_color};
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    font-family: 'Iosevka Web';
                    color: ${text_color};
                ",
                    bg_color = bg_color,
                    text_color = text_color
                };

                let start_btn_style = css! {"
                    border: 4px solid ${box_border_color};
                    width: 300px; height: 100px;
                    font-size: 50px;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: 0.3s;
                    user-select: none;
                    :hover {
                        opacity: 0.8;
                    }
                ",
                    box_border_color = box_border_color
                };

                let start_btn_onclick = ctx.link().callback(move |e| Self::Message::Begin);
                let items = match inner {
                    NotBegunState::ModeSelection | NotBegunState::MPCreateJoinLobby => {
                        unreachable!("caught before")
                    }
                    NotBegunState::Initial => {
                        html! {
                            <div onclick={start_btn_onclick} class={ start_btn_style }>{ "Start" }</div>
                        }
                    }
                    NotBegunState::Ended => {
                        html! {
                            <>
                                <p class={css!{"font-size: 35px;"}}>{"Game over!"}</p>
                                <div onclick={start_btn_onclick} class={ start_btn_style }>{ "Try again" }</div>
                            </>
                        }
                    }
                };

                html! {
                    <div ref={self.refs.canvas_overlay.clone()} class={canvas_overlay_style}>
                        { items }
                    </div>
                }
            }
        };

        let body = match self.state {
            State::NotBegun { inner }
                if inner == NotBegunState::ModeSelection
                    || inner == NotBegunState::MPCreateJoinLobby =>
            {
                html! {
                    <>
                    <Global css={"background-color: black;"}/>
                    {main_area}
                    </>
                }
            }
            _ => {
                html! {
                    <div class={ wrapper_style }>
                        { main_area }
                        <div class={ panel_style }>
                            { btns }
                        </div>
                    </div>
                }
            }
        };

        html! {
            <>
                <Global css={global_style}/>
                <PageTitle title={"Snake"}/>
                { body }
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        if self.canvas_requires_fit {
            ctx.link().send_message(Self::Message::FitCanvasImmediately);
        }

        match self.state {
            State::Begun { .. } => {
                let r = self.refs.canvas_renderer();
                r.set_stroke_style(box_border_color);
                r.set_line_join("round");
                r.set_fill_style(bg_color);

                // assert!(self.refs.is_canvas_fit(self.state));
                let cd = self.refs.canvas_dimensions();
                r.fill_rect(domain::Boundaries {
                    min: domain::Pos::new(0, 0),
                    max: domain::Pos::new(cd.width as i32, cd.height as i32),
                });

                self.draw_snake(&r);
                self.draw_foods(&r);
                self.draw_boundaries(&r);
            }
            State::NotBegun { .. } => {}
        }
        // console::log!("random int (0..1)", rand_from_iterator(0..1));
        // console::log!("random int (0..2)", rand_from_iterator(0..2));
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Nothing => false,
            Self::Message::WindowLoaded => {
                ctx.link().send_message(Self::Message::FitCanvasImmediately);
                false
            }
            Self::Message::WindowResized => {
                ctx.link().send_message(Self::Message::FitCanvasImmediately);
                false
            }
            Self::Message::FitCanvasImmediately => match self.state {
                State::Begun { .. } => {
                    self.canvas_requires_fit = false;
                    self.refs.fit_canvas();
                    self.px_scale =
                        calc_px_scale(self.refs.canvas_dimensions(), self.domain.boundaries);
                    true
                }
                State::NotBegun { .. } => false,
            },
            Self::Message::Advance => {
                let game_over = || {
                    ctx.link()
                        .send_message(Self::Message::StateChange(State::NotBegun {
                            inner: NotBegunState::Ended,
                        }));
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
            Self::Message::Restart
            | Self::Message::DirectionChange(_)
            | Self::Message::CameraToggle
            | Self::Message::PauseUnpause
                if matches!(self.state, State::NotBegun { .. }) =>
            {
                console::log!("ignoring event: State::NotBegun");
                false
            }
            Self::Message::Restart => {
                // drop old by replacement
                self.advance_interval.reset();
                // reset domain items
                self.domain = Domain::default();

                Refs::fire_btn_active(self.refs.restart_btn_el());

                true
            }
            Self::Message::DirectionChange(direction) => {
                if self.domain.snake.set_direction(direction).is_err() {
                    console::log!("cannot move into the opposite direction")
                }

                let btn = self.refs.ctrl_btn_el(direction);
                Refs::fire_btn_active(btn);

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
                Refs::fire_btn_active(self.refs.camera_btn_el());
                false
            }
            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Snake");
                self.theme_ctx.set(theme_ctx);
                true
            }
            Self::Message::StateChange(new_state) if new_state == self.state => false,
            Self::Message::StateChange(new_state @ State::Begun { paused }) => {
                if paused {
                    self.advance_interval.stop();
                } else {
                    self.advance_interval.start();
                }

                match self.state {
                    State::Begun { .. } => {}
                    State::NotBegun { .. } => {
                        self.canvas_requires_fit = true;
                    }
                }

                self.state = new_state;
                true
            }
            Self::Message::StateChange(new_state @ State::NotBegun { .. }) => {
                self.advance_interval.stop();
                self.domain = Domain::default();
                assert_eq!(self.canvas_requires_fit, false);
                self.state = new_state;
                true
            }
            Self::Message::Begin => {
                ctx.link()
                    .send_message(Self::Message::StateChange(State::Begun { paused: false }));
                false
            }
            Self::Message::PauseUnpause => match self.state {
                State::Begun { paused } => {
                    ctx.link()
                        .send_message(Self::Message::StateChange(State::Begun { paused: !paused }));
                    // TODO maybe move to StateChange
                    Refs::fire_btn_active(self.refs.pause_btn_el());
                    false
                }
                State::NotBegun { .. } => {
                    assert!(false, "game has not begun");
                    false
                }
            },
            Self::Message::ToMenu => {
                ctx.link()
                    .send_message(Self::Message::StateChange(State::NotBegun {
                        inner: NotBegunState::Initial,
                    }));
                false
            }
        }
    }
}

impl Snake {
    pub fn transform_pos(&self, pos: domain::Pos) -> TransformedPos {
        let pos = TransformedPos::from(self.adjust_algo.apply(pos)) * self.px_scale;

        match self.camera {
            Camera::MouthCentered => {
                // center camera to the mouth
                //
                // position of the mouth after the same transformations as of 'pos'
                let adjusted_mouth =
                    TransformedPos::from(self.adjust_algo.apply(self.domain.snake.mouth()))
                        * self.px_scale;
                // target position - center of the canvas
                // assert!(self.refs.is_canvas_fit(self.state));
                let cd = self.refs.canvas_dimensions() / 2.0;
                let to_center_x = cd.width - adjusted_mouth.x;
                let to_center_y = cd.height - adjusted_mouth.y;

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
                ) * self.px_scale;
                // assert!(self.refs.is_canvas_fit(self.state));
                let cd = self.refs.canvas_dimensions() / 2.0;
                let to_center_x = cd.width - adjusted_boundaries_center.x;
                let to_center_y = cd.height - adjusted_boundaries_center.y;

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

    fn draw_snake(&self, r: &CanvasRenderer) {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        let snake_body_width = SNAKE_BODY_WIDTH * self.px_scale;

        r.set_line_width(snake_body_width);
        let pos = self.transform_pos(self.domain.snake.iter_vertices().next().unwrap());
        r.begin_path();
        r.move_to(pos);
        for pos in self
            .domain
            .snake
            .iter_vertices()
            .skip(1)
            .map(|v| self.transform_pos(v))
        {
            r.line_to(pos);
        }
        r.stroke();
        r.close_path();

        // round head
        let pos = self.transform_pos(self.domain.snake.mouth());
        r.begin_path();
        r.cirle(pos, snake_body_width / 2.);
        r.set_fill_style(box_border_color);
        r.fill();
        r.close_path();

        r.begin_path();
        r.cirle(pos, (snake_body_width / 2.) * 0.9);
        r.set_fill_style(bg_color);
        r.fill();
        r.close_path();

        // round tail
        let pos = self.transform_pos(self.domain.snake.tail_end());
        r.begin_path();
        r.cirle(pos, snake_body_width / 2.);
        r.set_fill_style(box_border_color);
        r.fill();
        r.close_path();
    }

    fn draw_foods(&self, r: &CanvasRenderer) {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        for food in self.domain.foods.as_ref() {
            let pos = self.transform_pos(food.pos);
            r.begin_path();
            r.cirle(pos, FOOD_DIAMETER * self.px_scale / 2.);
            r.set_fill_style(box_border_color);
            r.fill();
            r.close_path();
        }
    }

    fn draw_boundaries(&self, r: &CanvasRenderer) {
        r.set_line_width(0.5 * self.px_scale);
        let pos = self.transform_pos(self.domain.boundaries.left_top());

        r.begin_path();
        r.move_to(pos);

        for pos in [
            self.domain.boundaries.right_top(),
            self.domain.boundaries.right_bottom(),
            self.domain.boundaries.left_bottom(),
            self.domain.boundaries.left_top(),
        ] {
            let pos = self.transform_pos(pos);
            r.line_to(pos);
        }
        r.close_path();
        r.stroke();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum NotBegunState {
    ModeSelection,
    MPCreateJoinLobby,
    Initial,
    Ended,
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Begun { paused: bool },
    NotBegun { inner: NotBegunState },
}

pub enum Camera {
    MouthCentered,
    BoundariesCentered,
}

enum AdjustAlgoChoice {
    None,
    // redundant at least when "camera" is centered to the mouth
    For4thQuadrant,
}

impl AdjustAlgoChoice {
    fn into_algo(&self, domain: &Domain) -> AdjustAlgo {
        match ADJUST_ALGO {
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
        }
    }
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
pub struct BtnRefs {
    menu_btn_ref: NodeRef,
    camera_btn_ref: NodeRef,
    restart_btn_ref: NodeRef,
    pause_btn_ref: NodeRef,
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
    canvas_overlay: NodeRef,
    canvas_ref: NodeRef,
    ctrl_brn_refs: CtrlBtnRefs,
    btn_refs: BtnRefs,
}

impl Refs {
    fn canvas_el(&self) -> HtmlCanvasElement {
        self.canvas_ref.clone().cast::<HtmlCanvasElement>().unwrap()
    }

    fn canvas_rendering_ctx(&self) -> CanvasRenderingContext2d {
        self.canvas_el()
            .get_context("2d")
            .unwrap()
            .unwrap()
            .unchecked_into::<CanvasRenderingContext2d>()
    }

    fn canvas_renderer(&self) -> CanvasRenderer {
        CanvasRenderer::from(self.canvas_rendering_ctx())
    }

    fn canvas_dimensions(&self) -> Dimensions {
        let canvas = self.canvas_el();
        Dimensions {
            width: canvas.width(),
            height: canvas.height(),
        }
    }

    fn is_canvas_fit(&self, state: State) -> bool {
        match state {
            State::Begun { .. } => self.canvas_dimensions() == canvas_target_dimensions(),
            State::NotBegun { .. } => true,
        }
    }

    fn set_canvas_dimensions(&self, dims: Dimensions) {
        let canvas = self.canvas_el();
        canvas.set_height(dims.height);
        canvas.set_width(dims.width);
        // console::log!("canvas resized to:", format!("{:?}", dims));
    }

    fn fit_canvas(&self) {
        let cd = canvas_target_dimensions();
        self.set_canvas_dimensions(cd);
    }

    fn ctrl_btn_el(&self, direction: domain::Direction) -> HtmlElement {
        self.ctrl_brn_refs
            .from_direction(direction)
            .clone()
            .cast::<HtmlElement>()
            .unwrap()
    }

    fn restart_btn_el(&self) -> HtmlElement {
        self.btn_refs
            .restart_btn_ref
            .clone()
            .cast::<HtmlElement>()
            .unwrap()
    }

    fn camera_btn_el(&self) -> HtmlElement {
        self.btn_refs
            .camera_btn_ref
            .clone()
            .cast::<HtmlElement>()
            .unwrap()
    }

    fn pause_btn_el(&self) -> HtmlElement {
        self.btn_refs
            .pause_btn_ref
            .clone()
            .cast::<HtmlElement>()
            .unwrap()
    }

    fn fire_btn_active(btn: HtmlElement) {
        if !btn.class_name().contains("active_btn") {
            btn.set_class_name(&format!("active_btn {}", btn.class_name()));
        }

        gloo_timers::callback::Timeout::new(200, move || {
            btn.set_class_name(&btn.class_name().replace("active_btn ", ""));
        })
        .forget();
    }
}

pub struct Listeners {
    kb_listener: EventListener,
    window_load_listener: EventListener,
    window_resize_listener: EventListener,
}

impl Listeners {
    fn init(link: Scope<Snake>) -> Self {
        // this event is unreliable, might not be emmited
        let window_load_listener = {
            let link = link.clone();
            EventListener::new(&get_window(), "load", move |event| {
                console::log!("event: window load");
                link.send_message(SnakeMsg::WindowLoaded);
            })
        };

        let window_resize_listener = {
            let link = link.clone();
            EventListener::new(&get_window(), "resize", move |event| {
                console::log!("event: window resize");
                link.send_message(SnakeMsg::WindowResized);
            })
        };

        let kb_listener = {
            let link = link.clone();
            EventListener::new_with_options(
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
                        PauseUnpause,
                    }
                    use KeyBoardEvent::*;

                    let kb_event = match event.key().as_str() {
                        "ArrowUp" => DirectionChange(domain::Direction::Up),
                        "ArrowDown" => DirectionChange(domain::Direction::Bottom),
                        "ArrowLeft" => DirectionChange(domain::Direction::Left),
                        "ArrowRight" => DirectionChange(domain::Direction::Right),
                        "r" | "R" => Restart,
                        "c" | "C" => CameraToggle,
                        "p" | "P" => PauseUnpause,
                        _ => None,
                    };

                    let message = match kb_event {
                        DirectionChange(direction) => SnakeMsg::DirectionChange(direction),
                        Restart => SnakeMsg::Restart,
                        None => SnakeMsg::Nothing,
                        CameraToggle => SnakeMsg::CameraToggle,
                        PauseUnpause => SnakeMsg::PauseUnpause,
                    };
                    link.send_message(message)
                },
            )
        };

        Self {
            kb_listener,
            window_load_listener,
            window_resize_listener,
        }
    }
}

pub struct Domain {
    snake: domain::Snake,
    foods: domain::Foods,
    boundaries: domain::Boundaries,
}

struct DomainDefaults;

impl DomainDefaults {
    fn snake() -> domain::Snake {
        let rand_section_len = || rand_from_iterator(3..5);

        let initial_pos = domain::Pos::new(-2, 2);

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
    fn default() -> Self {
        let snake = DomainDefaults::snake();
        let boundaries = DomainDefaults::boundaries(&snake);

        let food_average = ((MAP_BOUNDARIES_X * MAP_BOUNDARIES_Y) as f64 * 0.5);
        Self {
            foods: DomainDefaults::foods(
                rand_from_iterator(((food_average * 0.9) as i32)..((food_average * 1.1) as i32)),
                boundaries,
                snake.iter_vertices(),
            ),
            boundaries,
            snake,
        }
    }
}

struct SnakeAdvanceInterval {
    link: Scope<Snake>,
    _handle: Option<Interval>,
}

impl SnakeAdvanceInterval {
    fn create(link: Scope<Snake>) -> Self {
        Self {
            link,
            _handle: None,
        }
    }

    fn reset(&mut self) {
        let link = self.link.clone();
        let new_handle = || {
            Interval::new(SNAKE_ADVANCE_INTERVAL, move || {
                link.send_message(if PAUSED {
                    SnakeMsg::Nothing
                } else {
                    SnakeMsg::Advance
                })
            })
        };
        self._handle = Some(new_handle());
    }

    fn start(&mut self) {
        // don't reset if already started
        if let None = self._handle {
            self.reset();
        }
    }

    fn stop(&mut self) {
        self._handle = None;
    }
}

fn get_window() -> Window {
    web_sys::window().unwrap()
}

fn window_dimensions() -> Dimensions {
    Dimensions::from(get_window())
}

fn canvas_target_dimensions() -> Dimensions {
    let Dimensions { width, height } = window_dimensions();

    Dimensions {
        width: width - PANEL_PX_WIDTH,
        height,
    }
}

fn get_document() -> Document {
    let window = get_window();
    window.document().unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

struct CanvasRenderer {
    context: CanvasRenderingContext2d,
}

impl From<CanvasRenderingContext2d> for CanvasRenderer {
    fn from(value: CanvasRenderingContext2d) -> Self {
        Self { context: value }
    }
}

impl AsRef<CanvasRenderingContext2d> for CanvasRenderer {
    fn as_ref(&self) -> &CanvasRenderingContext2d {
        &self.context
    }
}

impl CanvasRenderer {
    fn cirle(&self, TransformedPos { x, y }: TransformedPos, radius: f64) {
        self.as_ref()
            .arc(x, y, radius, 0f64, 2.0 * std::f64::consts::PI)
            .unwrap();
    }

    fn begin_path(&self) {
        self.as_ref().begin_path();
    }

    fn close_path(&self) {
        self.as_ref().close_path();
    }

    fn stroke(&self) {
        self.as_ref().stroke();
    }

    fn fill(&self) {
        self.as_ref().fill();
    }

    fn move_to(&self, TransformedPos { x, y }: TransformedPos) {
        self.as_ref().move_to(x, y);
    }

    fn line_to(&self, TransformedPos { x, y }: TransformedPos) {
        self.as_ref().line_to(x, y);
    }

    fn set_fill_style(&self, value: &str) {
        self.as_ref().set_fill_style(&JsValue::from_str(value));
    }

    fn set_stroke_style(&self, value: &str) {
        self.as_ref().set_stroke_style(&JsValue::from_str(value));
    }

    fn set_line_join(&self, value: &str) {
        self.as_ref().set_line_join(value);
    }

    fn set_line_width(&self, value: f64) {
        self.as_ref().set_line_width(value);
    }

    fn fill_rect(&self, boundaries: domain::Boundaries) {
        let TransformedPos { x: min_x, y: min_y } = boundaries.min.into();
        let TransformedPos { x: max_x, y: max_y } = boundaries.max.into();
        self.as_ref().fill_rect(min_x, min_y, max_x, max_y);
    }
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

fn calc_px_scale(canvas_dimensions: Dimensions, boundaries: domain::Boundaries) -> f64 {
    // scale game to fully fit into space available to canvas
    // when camera is Camera::BoundariesCentered

    // add ones to for bounds strokes to be inside space boundaries
    // can be chosen more accurate value in px
    f64::min(
        canvas_dimensions.height as f64 / (boundaries.height() + 1) as f64,
        canvas_dimensions.width as f64 / (boundaries.width() + 1) as f64,
    )
}
