#![allow(unused, non_upper_case_globals)]
use crate::components::imports::*;
use futures::SinkExt;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_timers::callback::Interval;
use std::collections::HashSet;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};
use yew::html::Scope;

use interfacing::snake::{
    lobby_state::{LobbyPrep, LobbyRunning},
    JoinLobbyDecline, LobbyName, LobbyState, PinnedMessage, UserName, WsClientMsg, WsMsg,
    WsServerMsg,
};

use super::styles;

type ClientMsg = WsMsg<interfacing::snake::WsClientMsg>;
type ServerMsg = WsMsg<interfacing::snake::WsServerMsg>;

use domain::Domain;
use interfacing::snake_domain as domain;

const PAUSED: bool = false;
const NOT_BEGUN_STATE: NotBegunState = NotBegunState::ModeSelection;

// const STATE: State = State::Begun { paused: false };

const CAMERA: Camera = Camera::BoundariesCentered;

const SNAKE_ADVANCE_INTERVAL: u32 = 450; // in milliseconds

const MAP_BOUNDARIES_X: i32 = 10;
const MAP_BOUNDARIES_Y: i32 = 10;

const PANEL_PX_WIDTH: u32 = 350;

const SNAKE_BODY_WIDTH: f64 = 0.9;
const FOOD_DIAMETER: f64 = 0.6;

#[derive(Debug, Default)]
pub struct WsState {
    user_name: Option<UserName>,
    // if both Server/Client default to None this can be removed
    //
    // can be useful for debugging
    // if on connect Server would set a random user_name to the connection
    synced_user_name: bool,
    joined_lobby_name: Option<LobbyName>,
    joined_lobby_state: Option<interfacing::snake::LobbyState>,
}

pub struct Snake {
    state: State,

    // true if state changed from NotBegun to Begun
    canvas_requires_fit: bool,

    camera: Camera,

    refs: Refs,
    listeners: Listeners,

    ws_sink: tokio::sync::mpsc::UnboundedSender<ClientMsg>,
    ws_state: WsState,
    acknowledgeable_messages: AcknowledgeableMessages,

    theme_ctx: ThemeCtxSub,
}

#[derive(Default, derived_deref::Deref, derived_deref::DerefMut)]
struct AcknowledgeableMessages(
    std::collections::HashMap<interfacing::snake::MsgId, interfacing::snake::WsClientMsg>,
);

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
    RedirectToLobby { lobby_name: LobbyName },
    StateChange(State),
    Begin,
    PauseUnpause,
    ToMenu,
    // LeaveLobby,
    WsSend(ClientMsg),
    WsRecv(ServerMsg),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or(NOT_BEGUN_STATE)]
    pub state: NotBegunState,
}

impl Component for Snake {
    type Message = SnakeMsg;
    type Properties = Props;

    #[allow(unused_variables)]
    fn create(ctx: &Context<Self>) -> Self {
        let state = ctx.props().state.clone();

        let ws_sink = {
            use crate::ws::imports::*;

            fn read_stream(stream: SplitStream<WebSocket>) -> impl Stream<Item = SnakeMsg> {
                stream.map(|i| match i {
                    Ok(msg) => match msg {
                        Message::Text(text) => {
                            let msg = serde_json::from_str::<ServerMsg>(&text).unwrap(); // TODO handle unwrap
                            SnakeMsg::WsRecv(msg)
                        }
                        Message::Bytes(_) => unimplemented!(),
                    },
                    // TODO impl reconnect
                    Err(gloo_net::websocket::WebSocketError::ConnectionClose(e)) => {
                        console::log!(format!("{} {} {}", e.code, e.reason, e.was_clean));
                        SnakeMsg::Nothing
                    }
                    Err(gloo_net::websocket::WebSocketError::ConnectionError) => {
                        console::log!("! read channel ConnectionError");
                        SnakeMsg::Nothing
                    }
                    Err(gloo_net::websocket::WebSocketError::MessageSendError(_)) => unreachable!(),
                    Err(_) => unreachable!(),
                })
            }

            async fn write_stream(
                mut stream: SplitSink<WebSocket, Message>,
                mut r: tokio::sync::mpsc::UnboundedReceiver<ClientMsg>,
            ) -> SnakeMsg {
                while let Some(msg) = r.recv().await {
                    let msg = Message::Text(serde_json::to_string(&msg).unwrap());
                    stream.send(msg).await.unwrap(); // TODO handle
                }
                console::log!("! write channel closed");
                SnakeMsg::Nothing
            }

            // NOTE do not rewrite with futures::channel::mpsc,
            // async send makes calling dirtier locally
            let (s, r) = tokio::sync::mpsc::unbounded_channel::<ClientMsg>();

            let url = crate::ws::prepare_relative_url("/api/snake/ws");
            let ws = WebSocket::open(&url).unwrap();

            let (w_ws, r_ws) = ws.split();
            ctx.link().send_stream(read_stream(r_ws));
            ctx.link().send_future(write_stream(w_ws, r));

            s
        };

        Self {
            state: State::NotBegun { inner: state },

            canvas_requires_fit: false,

            camera: CAMERA,

            refs: Default::default(),
            listeners: Listeners::init(ctx.link().clone()),

            ws_sink,
            ws_state: Default::default(),
            acknowledgeable_messages: Default::default(),

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

        // HERE
        let btn_style = css! {"
            border: 2px solid ${box_border_color};
            width: 80px; height: 20px;
            color: ${text_color};
            display: inline-block;
            padding: 7px 5px;
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
                State::BegunSingleplayer { .. } | State::BegunMultiplayer { .. } => {
                    let direction_btn = |text: &str, direction| {
                        let direction_btn_onlick = |direction| {
                            ctx.link()
                                .callback(move |_| Self::Message::DirectionChange(direction))
                        };

                        let direction_btn_style = css! {"margin: 2px 1px;"};

                        html! {
                            <div
                                ref={ self.refs.ctrl_brn_refs.from_direction(direction) }
                                class={ vec![btn_style.clone(), direction_btn_style, styles::btn_style()] }
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
                            <div ref={self.refs.btn_refs.menu_btn_ref.clone()} class={ vec![margin_bottom_btn_style.clone(), btn_style.clone(), styles::btn_style()] } onclick={to_main_screen_btn_onclick}>{ "Menu" }</div>

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

                            <div ref={self.refs.btn_refs.camera_btn_ref.clone()} class={ vec![margin_top_btn_style.clone(), btn_style.clone(), styles::btn_style()] } onclick={camera_btn_onclick}>{ "Camera (C)" }</div>
                            <div ref={self.refs.btn_refs.restart_btn_ref.clone()} class={ vec![margin_top_btn_style.clone(), btn_style.clone(), styles::btn_style()] } onclick={restart_btn_onclick}>{ "Restart (R)" }</div>
                            <div ref={self.refs.btn_refs.pause_btn_ref.clone()} class={ vec![margin_top_btn_style.clone(), btn_style.clone(), styles::btn_style()] } onclick={pause_btn_onclick}>{ "Pause (P)" }</div>
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

        use interfacing::snake::PinnedMessage;

        let main_area = 'main_area: {
            match &self.state {
                State::BegunSingleplayer { .. } | State::BegunMultiplayer { .. } => {
                    html! { <canvas ref={self.refs.canvas_ref.clone() }></canvas> }
                }
                State::NotBegun {
                    inner: NotBegunState::MPLobbyList { lobbies },
                } => match lobbies {
                    None => {
                        ctx.link().send_message(SnakeMsg::WsSend(
                            "lobby-list".pinned_msg(WsClientMsg::LobbyList),
                        ));

                        html! {"Loading Lobby list"}
                    }
                    Some(lobbies) => {
                        let onclick = {
                            let link = ctx.link().clone();
                            move |name: String| {
                                let link = link.clone();
                                move |_| {
                                    link.send_message(SnakeMsg::RedirectToLobby {
                                        lobby_name: name.clone(),
                                    })
                                }
                            }
                        };

                        let lobbies = lobbies
                            .into_iter()
                            .map(|lobby| html! { <h1 onclick={onclick(lobby.name.clone())}>{ &lobby.name }</h1> })
                            .collect::<Html>();

                        html! {
                            <>
                            { "Lobbies" }
                            { lobbies }
                            </>
                        }
                    }
                },
                State::NotBegun {
                    inner: NotBegunState::MPSetUsername { next_state },
                } => {
                    if self.ws_state.synced_user_name {
                        match &self.ws_state.user_name {
                            None => {
                                // enter user_name

                                let user_name_ref = NodeRef::default();

                                let onsubmit = {
                                    let user_name_ref = user_name_ref.clone();

                                    ctx.link().callback(move |event: SubmitEvent| {
                                        event.prevent_default();

                                        let join_as = user_name_ref
                                            .cast::<HtmlInputElement>()
                                            .unwrap()
                                            .value();

                                        SnakeMsg::WsSend(
                                            "set-username"
                                                .pinned_msg(WsClientMsg::SetUserName(join_as)),
                                        )
                                    })
                                };

                                html! {
                                    <>
                                    {"Join as..."}
                                    <form {onsubmit} method="post">
                                        <input type="text" ref={user_name_ref}/>
                                    </form>
                                    </>
                                }
                            }
                            Some(user_name) => {
                                let msg = SnakeMsg::StateChange(State::NotBegun {
                                    inner: *next_state.clone(),
                                });

                                ctx.link().send_message(msg);

                                html! {}
                            }
                        }
                    } else {
                        match &self.ws_state.user_name {
                            None => {
                                let msg = SnakeMsg::WsSend(
                                    "ask-username".pinned_msg(WsClientMsg::UserName),
                                );

                                ctx.link().send_message(msg);

                                // should reload this state, but jump to another branch
                                html! {}
                            }
                            Some(_) => unreachable!("cannot be set without sync"),
                        }
                    }
                }
                State::NotBegun {
                    inner: s @ NotBegunState::MPLobby { state },
                } => {
                    // at this point set UserName is required to continue
                    //
                    if let None = self.ws_state.user_name {
                        let msg = Self::Message::StateChange(State::NotBegun {
                            inner: NotBegunState::MPSetUsername {
                                next_state: Box::new(s.clone()),
                            },
                        });
                        ctx.link().send_message(msg);

                        return html! {};
                    }

                    use MPLobbyState::*;
                    match state {
                        ToJoin { lobby_name } => {
                            let msg = SnakeMsg::WsSend(
                                "join-lobby".pinned_msg(WsClientMsg::JoinLobby(lobby_name.clone())),
                            );

                            ctx.link().send_message(msg);

                            html! { "Joining..." }
                            // unimplemented!()
                        }
                        Joined => {
                            let ls = self.ws_state.joined_lobby_state.as_ref().expect("to exist");
                            let lobby_name =
                                self.ws_state.joined_lobby_name.as_ref().expect("to exist");

                            // TODO make request to server
                            pub async fn get_lobby(
                                name: String,
                            ) -> Result<interfacing::snake::GetLobby, ()>
                            {
                                let response = Request::get(&format!("/api/snake/lobby/{}", name))
                                    .send()
                                    .await
                                    .unwrap(); // TODO handle

                                match response.status() {
                                    200 => Ok(response
                                        .json::<interfacing::snake::GetLobby>()
                                        .await
                                        .unwrap()), // TODO handle
                                    _ => unimplemented!(), // TODO handle
                                }
                            }

                            // {
                            //     let name = name.clone();
                            //     ctx.link().send_future(async move {
                            //         match get_lobby(name.clone()).await {
                            //             Ok(lobby) => unimplemented!(),
                            //             Err(_e) => Self::Message::Nothing,
                            //         }
                            //     });
                            // }

                            let block = match ls {
                                LobbyState::Prep(LobbyPrep { participants }) => {
                                    let onclick = ctx.link().callback(move |e| {
                                        Self::Message::WsSend("vote-start".pinned_msg(
                                            interfacing::snake::WsClientMsg::VoteStart(true),
                                        ))
                                    });

                                    let part = participants
                                        .into_iter()
                                        .map(|p| {
                                            html! {
                                                <>
                                                <h3>{&p.user_name} {" voted: "} {p.vote_start} </h3>
                                                </>
                                            }
                                        })
                                        .collect::<Html>();

                                    html! {
                                        <>
                                        {part}
                                        <button {onclick}> { "Vote start" } </button>
                                        </>
                                    }
                                }

                                LobbyState::Running(LobbyRunning {
                                    counter,
                                    player_counter,
                                    domain,
                                }) => {
                                    ctx.link().send_message(SnakeMsg::StateChange(
                                        State::BegunMultiplayer {
                                            domain: domain.clone(),
                                            px_scale: calc_px_scale(&domain.boundaries),
                                        },
                                    ));

                                    html! {
                                        <>
                                        <h2>{"Player count: "}{player_counter}</h2>
                                        <h1>{"Running: "} { counter }</h1>
                                        </>
                                    }
                                }

                                LobbyState::Terminated => {
                                    // currently this state does not reach a client
                                    html! {<h1>{ "Terminated, you should have been redirected" }</h1>}
                                }
                            };

                            let leave_lobby = {
                                let onclick =
                                    ctx.link().callback(move |e| {
                                        Self::Message::WsSend("leave-lobby".pinned_msg(
                                            interfacing::snake::WsClientMsg::LeaveLobby,
                                        ))
                                    });

                                html! { <button {onclick}>{ "Leave" }</button> }
                            };

                            html! {
                                <>
                                <p></p>
                                { leave_lobby }
                                <h2>{ "Joined "} { lobby_name } { " as " } { self.ws_state.user_name.as_ref().unwrap() } </h2>
                                { block }
                                </>
                            }
                        }

                        JoinError {
                            lobby_name,
                            message,
                        } => {
                            html! { <> {"Join "} { lobby_name } {" error: "} { message }  </> }
                        }
                    }
                }
                State::NotBegun {
                    inner: NotBegunState::MPCreateLobby,
                } => {
                    let name_ref = NodeRef::default();

                    async fn post_lobby(
                        form: &interfacing::snake::CreateLobby,
                    ) -> request::SendResult {
                        Request::post("/api/snake/lobby")
                            .json(&form)
                            .unwrap()
                            .send()
                            .await
                    }

                    let onsubmit = {
                        let name_ref = name_ref.clone();

                        ctx.link().callback_future(move |event: SubmitEvent| {
                            event.prevent_default();

                            let name = name_ref.cast::<HtmlInputElement>().unwrap().value();

                            let form = interfacing::snake::CreateLobby { name: name.clone() };

                            async move {
                                console::log!(format!("submitting: {:?}", form));
                                let r = post_lobby(&form).await.unwrap();
                                r.log_status();

                                match r.status() {
                                    200 => {
                                        console::log!("lobby created");

                                        Self::Message::RedirectToLobby { lobby_name: name }
                                        // Self::Message::StateChange(State::NotBegun {
                                        //     inner: NotBegunState::MPPrejoinLobby {
                                        //         lobby_name: form.name,
                                        //     },
                                        // })
                                    }
                                    // TODO handle errors for validation
                                    409 => {
                                        web_sys::window().unwrap().alert_with_message(
                                            "Lobby with this name already exists",
                                        );
                                        Self::Message::Nothing
                                    }
                                    _ => unimplemented!(),
                                }
                            }
                        })
                    };

                    let style = vec![css! {"height:100vh;"}, styles::centered_column_items()];

                    html! {
                        <div class={style}>
                        <h1>{ "Enter new lobby name:" }</h1>
                        <form {onsubmit} method="post">
                            <input class={styles::input_style()} type="text" ref={name_ref}/>
                        </form>
                        </div>
                    }
                }
                State::NotBegun {
                    inner: NotBegunState::ModeSelection,
                } => {
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

                    let btn_style = vec![
                        css! {
                            "border-color: ${box_border_color};
                            margin-bottom: 50px;
                            ",
                            box_border_color = box_border_color
                        },
                        styles::big_btn_style(),
                        styles::btn_style(),
                    ];

                    html! {
                        <div class={vec![css!{"height: 100vh;"}, styles::centered_column_items()]}>
                            <div onclick={ sp_onclick } class={btn_style.clone()}>{ "Singleplayer" }</div>
                            <div onclick={ mp_onclick } class={btn_style.clone()}>{ "Multiplayer" }</div>
                        </div>
                    }
                }
                State::NotBegun {
                    inner: NotBegunState::MPCreateJoinLobby,
                } => {
                    let create_onclick = ctx.link().callback(move |e| {
                        Self::Message::StateChange(State::NotBegun {
                            inner: NotBegunState::MPCreateLobby,
                        })
                    });

                    let join_onclick = ctx.link().callback(move |e| {
                        Self::Message::StateChange(State::NotBegun {
                            inner: NotBegunState::MPLobbyList { lobbies: None },
                        })
                    });

                    let btn_style = vec![
                        css! {
                            "border-color: ${box_border_color};
                            margin-bottom: 50px;
                            ",
                            box_border_color = box_border_color
                        },
                        styles::big_btn_style(),
                        styles::btn_style(),
                    ];

                    html! {
                        <div class={vec![css!{"height: 100vh;"}, styles::centered_column_items()]}>
                            <div onclick={create_onclick} class={btn_style.clone()}>{ "Create server" }</div>
                            <div onclick={join_onclick} class={btn_style.clone()}>{ "Join server" }</div>
                        </div>
                    }
                }
                State::NotBegun { inner } => {
                    let canvas_overlay_style = css! {"
                        height: 100vh;
                        width: calc(100% - 350px);
                        background-color: ${bg_color};
                        font-family: 'Iosevka Web';
                        color: ${text_color};
                    ",
                        bg_color = bg_color,
                        text_color = text_color
                    };

                    let btn_style = vec![
                        css! {
                            "border-color: ${box_border_color};",
                            box_border_color = box_border_color
                        },
                        styles::big_btn_style(),
                        styles::btn_style(),
                    ];

                    let start_btn_onclick = ctx.link().callback(move |e| Self::Message::Begin);
                    let items = match inner {
                        NotBegunState::Initial => {
                            html! {
                                <div onclick={start_btn_onclick} class={ btn_style.clone() }>{ "Start" }</div>
                            }
                        }
                        NotBegunState::Ended => {
                            html! {
                                <>
                                    <p class={css!{"font-size: 35px;"}}>{"Game over!"}</p>
                                    <div onclick={start_btn_onclick} class={ btn_style.clone() }>{ "Try again" }</div>
                                </>
                            }
                        }
                        _ => {
                            unreachable!("caught before")
                        }
                    };

                    html! {
                        <div ref={self.refs.canvas_overlay.clone()}
                            class={vec![styles::centered_column_items(), canvas_overlay_style]}>
                            { items }
                        </div>
                    }
                }
            }
        };

        let body = match &self.state {
            State::NotBegun {
                inner: NotBegunState::Initial | NotBegunState::Ended,
            }
            | State::BegunSingleplayer { .. }
            | State::BegunMultiplayer { .. } => {
                html! {
                    <div class={ wrapper_style }>
                        { main_area }
                        <div class={ panel_style }>
                            { btns }
                        </div>
                    </div>
                }
            }
            _ => {
                let global_style = css!(
                    "
                        body {
                            font-family: 'Iosevka Web';
                            background-color: ${bg_color};
                            color: ${text_color};
                        }
                    ",
                    bg_color = bg_color,
                    text_color = text_color,
                );

                html! {
                    <>
                    <Global css={global_style}/>
                    {main_area}
                    </>
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
        if first_render {
            // ctx.link().send_message(SnakeMsg::WsSend(
            //     WsMsg::new(interfacing::snake::WsClientMsg::SetUserName(
            //         "phantie".into(),
            //     ))
            //     .id("test-set-username"),
            // ));

            // ctx.link().send_message(SnakeMsg::WsSend(
            //     WsMsg::new(interfacing::snake::WsClientMsg::UserName).id("test-query-username"),
            // ));

            // {
            //     // let (_, r) = futures::channel::mpsc::unbounded::<()>();

            //     ctx.link().send_future(async {
            //         sleep(2000).await;
            //         SnakeMsg::WsSend(interfacing::snake::Msg(
            //             "".into(),
            //             interfacing::snake::WsClientMsg::UserName("phantie".into()),
            //         ))
            //     });
            // }
        }

        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        if self.canvas_requires_fit {
            ctx.link().send_message(Self::Message::FitCanvasImmediately);
        }

        match &self.state {
            s @ (State::BegunSingleplayer { .. } | State::BegunMultiplayer { .. }) => {
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

                match s {
                    State::NotBegun { .. } => unreachable!(),
                    State::BegunSingleplayer {
                        snake,
                        foods,
                        boundaries,
                        px_scale,
                        ..
                    } => {
                        self.draw_snake(&r, snake, boundaries, false, *px_scale);
                        self.draw_foods(&r, foods, Some(snake), boundaries, *px_scale);
                        self.draw_boundaries(&r, boundaries, Some(snake), *px_scale);
                    }
                    State::BegunMultiplayer {
                        domain:
                            Domain {
                                snake,
                                other_snakes,
                                foods,
                                boundaries,
                            },
                        px_scale,
                    } => {
                        if let Some(snake) = snake {
                            self.draw_snake(&r, snake, boundaries, true, *px_scale);
                        }
                        for snake in other_snakes {
                            self.draw_snake(&r, snake, boundaries, false, *px_scale);
                        }
                        self.draw_foods(&r, foods, snake.as_ref(), boundaries, *px_scale);
                        self.draw_boundaries(&r, boundaries, snake.as_ref(), *px_scale);
                    }
                }
            }
            State::NotBegun { .. } => {}
        }
        // console::log!("random int (0..1)", rand_from_iterator(0..1));
        // console::log!("random int (0..2)", rand_from_iterator(0..2));
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::Nothing => false,

            // Self::Message::LeaveLobby => {
            //     ctx.link().send_message(Self::Message::WsSend(
            //         "leave-lobby".pinned_msg(interfacing::snake::WsClientMsg::LeaveLobby),
            //     ));
            //     false
            // }
            Self::Message::WsRecv(msg) => self.handle_received_message(ctx, msg),

            Self::Message::WsSend(msg) => {
                if let WsMsg(Some(id), msg) = &msg {
                    self.acknowledgeable_messages
                        .insert(id.clone(), msg.clone());
                }

                () = self.ws_sink.send(msg.clone()).unwrap(); // TODO handle
                console::log!(format!("sent: {:?}", msg));

                false
            }

            Self::Message::RedirectToLobby { lobby_name } => {
                ctx.link()
                    .send_message(Self::Message::StateChange(State::to_be_loaded_lobby(
                        lobby_name,
                    )));
                false
            }

            Self::Message::WindowLoaded => {
                ctx.link().send_message(Self::Message::FitCanvasImmediately);
                false
            }

            Self::Message::WindowResized => {
                ctx.link().send_message(Self::Message::FitCanvasImmediately);
                false
            }

            Self::Message::FitCanvasImmediately => match &mut self.state {
                State::BegunSingleplayer {
                    boundaries,
                    px_scale,
                    ..
                }
                | State::BegunMultiplayer {
                    domain: Domain { boundaries, .. },
                    px_scale,
                    ..
                } => {
                    self.canvas_requires_fit = false;
                    self.refs.fit_canvas();
                    *px_scale = calc_px_scale(boundaries);
                    true
                }
                State::NotBegun { .. } => false,
            },

            Self::Message::Advance => {
                match &mut self.state {
                    State::BegunSingleplayer {
                        snake,
                        foods,
                        boundaries,
                        ..
                    } => {
                        let game_over = || {
                            ctx.link()
                                .send_message(Self::Message::StateChange(State::NotBegun {
                                    inner: NotBegunState::Ended,
                                }));
                        };

                        match snake.advance(foods, &[], &boundaries) {
                            domain::AdvanceResult::Success => {
                                // when no food, replenish
                                if foods.empty() {
                                    *foods = DomainDefaults::foods(
                                        rand_from_iterator(10..15),
                                        *boundaries,
                                        snake.iter_vertices(),
                                    );
                                }
                            }
                            domain::AdvanceResult::BitYaSelf
                            | domain::AdvanceResult::OutOfBounds => game_over(),
                            domain::AdvanceResult::BitSomeone => unreachable!(),
                        }

                        true
                    }
                    _ => false,
                }
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
                match &mut self.state {
                    State::BegunSingleplayer {
                        snake,
                        boundaries,
                        foods,
                        advance_interval,
                        ..
                    } => {
                        let defaults = SPDomainDefaults::defaults();
                        *snake = defaults.snake;
                        *foods = defaults.foods;
                        *boundaries = defaults.boundaries;

                        // drop old by replacement
                        advance_interval.reset();
                    }
                    State::BegunMultiplayer { .. } => {
                        console::log!("Restart unsupported in MP");
                    }
                    State::NotBegun { .. } => {}
                }

                Refs::fire_btn_active(self.refs.restart_btn_el());

                true
            }

            Self::Message::DirectionChange(direction) => {
                match &mut self.state {
                    State::BegunSingleplayer { snake, .. } => {
                        if snake.set_direction(direction).is_err() {
                            console::log!("cannot move into the opposite direction")
                        }
                    }
                    State::BegunMultiplayer { .. } => {
                        ctx.link().send_message(SnakeMsg::WsSend(WsMsg(
                            None,
                            WsClientMsg::SetDirection(direction),
                        )));
                    }
                    State::NotBegun { .. } => {}
                }

                let btn = self.refs.ctrl_btn_el(direction);
                Refs::fire_btn_active(btn);

                false
            }

            Self::Message::CameraChange(camera) => self.change_camera(camera).is_ok(),

            Self::Message::CameraToggle => {
                // TODO rework camera management
                let next_camera = match self.camera {
                    Camera::MouthCentered => Camera::BoundariesCentered,
                    Camera::BoundariesCentered => Camera::MouthCentered,
                };
                Refs::fire_btn_active(self.refs.camera_btn_el());
                self.change_camera(next_camera).is_ok()
            }

            Self::Message::ThemeContextUpdate(theme_ctx) => {
                console::log!("WithTheme context updated from Snake");
                self.theme_ctx.set(theme_ctx);
                true
            }

            Self::Message::StateChange(new_state @ State::BegunMultiplayer { .. }) => {
                // TODO rework camera management
                if let State::BegunMultiplayer {
                    // no snake to control, no mouth to center to
                    domain: Domain { snake: None, .. },
                    ..
                } = &new_state
                {
                    self.change_camera(Camera::BoundariesCentered).unwrap_or(());
                }

                match self.state {
                    State::BegunSingleplayer { .. } => panic!("forbidden"),
                    State::BegunMultiplayer { .. } => {}
                    State::NotBegun { .. } => {
                        self.canvas_requires_fit = true;
                    }
                }

                self.state = new_state;
                true
            }

            Self::Message::StateChange(new_state @ State::BegunSingleplayer { .. }) => {
                match self.state {
                    State::BegunSingleplayer { .. } => {}
                    State::BegunMultiplayer { .. } => panic!("forbidden"),
                    State::NotBegun { .. } => {
                        self.canvas_requires_fit = true;
                    }
                }

                self.state = new_state;
                true
            }

            Self::Message::StateChange(new_state @ State::NotBegun { .. }) => {
                {
                    use crate::router::Route;

                    let route = match &new_state {
                        State::NotBegun { inner } => match inner {
                            NotBegunState::MPLobby { state } => {
                                match state {
                                    MPLobbyState::ToJoin { lobby_name } => {
                                        Some(Route::SnakeLobby {
                                            lobby_name: lobby_name.clone(),
                                        })
                                    }
                                    MPLobbyState::Joined | MPLobbyState::JoinError { .. } => {
                                        // should transition only from ToJoin, so ...
                                        None
                                    }
                                }
                            }
                            NotBegunState::MPCreateJoinLobby => Some(Route::SnakeCreateJoinLobby),
                            NotBegunState::MPCreateLobby => Some(Route::SnakeCreateLobby),
                            NotBegunState::MPLobbyList { .. } => Some(Route::SnakeLobbies),
                            NotBegunState::ModeSelection => Some(Route::Home),
                            _ => None,
                        },
                        State::BegunSingleplayer { .. } | State::BegunMultiplayer { .. } => None,
                    };

                    // console::log!("!!!!", format!("{route:?}"));

                    if let Some(route) = route {
                        let path = ctx.link().location().unwrap().path().to_string();
                        if route.to_path() != path {
                            let navigator = ctx.link().navigator().unwrap();
                            navigator.push(&route);
                        }
                    }
                }

                match &mut self.state {
                    State::NotBegun { .. } => {}
                    State::BegunSingleplayer {
                        snake,
                        foods,
                        boundaries,
                        advance_interval,
                        ..
                    } => {
                        let defaults = SPDomainDefaults::defaults();
                        *snake = defaults.snake;
                        *foods = defaults.foods;
                        *boundaries = defaults.boundaries;

                        advance_interval.stop();
                    }
                    State::BegunMultiplayer { .. } => {
                        // TODO implement leave lobby
                    }
                }

                assert_eq!(self.canvas_requires_fit, false);
                self.state = new_state;
                true
            }

            Self::Message::Begin => {
                let SPDomainDefaults {
                    snake,
                    foods,
                    boundaries,
                } = SPDomainDefaults::defaults();
                let mut advance_interval = SnakeAdvanceInterval::create(ctx.link().clone());
                advance_interval.start();

                ctx.link()
                    .send_message(Self::Message::StateChange(State::BegunSingleplayer {
                        snake,
                        foods,
                        boundaries,
                        px_scale: calc_px_scale(&boundaries),
                        advance_interval,
                    }));
                false
            }

            Self::Message::PauseUnpause => match &mut self.state {
                State::BegunSingleplayer {
                    advance_interval, ..
                } => {
                    if !advance_interval.paused() {
                        advance_interval.stop();
                    } else {
                        advance_interval.start();
                    }

                    Refs::fire_btn_active(self.refs.pause_btn_el());
                    false
                }

                State::BegunMultiplayer { .. } => {
                    console::log!("Pause unsupported in MP");
                    false
                }

                State::NotBegun { .. } => {
                    assert!(false, "game has not begun");
                    false
                }
            },

            Self::Message::ToMenu => {
                match self.state {
                    State::NotBegun { .. } => {}
                    State::BegunSingleplayer { .. } => {
                        ctx.link()
                            .send_message(Self::Message::StateChange(State::NotBegun {
                                inner: NotBegunState::Initial,
                            }));
                    }
                    State::BegunMultiplayer { .. } => {
                        ctx.link().send_message(Self::Message::WsSend(
                            "leave-lobby".pinned_msg(interfacing::snake::WsClientMsg::LeaveLobby),
                        ));
                    }
                }
                false
            }
        }
    }
}

pub fn calc_px_scale(boundaries: &domain::Boundaries) -> f64 {
    calculate_px_scale(canvas_target_dimensions(), boundaries)
}

impl Snake {
    pub fn change_camera(&mut self, camera: Camera) -> Result<(), ()> {
        if self.available_cameras().contains(&camera) {
            self.camera = camera;
            Ok(())
        } else {
            console::log!(format!("cannot apply Camera: {camera:?}"));
            Err(())
        }
    }

    pub fn available_cameras(&self) -> Vec<Camera> {
        // TODO move camera to Begun states
        match &self.state {
            State::NotBegun { .. } => vec![],
            State::BegunSingleplayer { snake, .. }
            | State::BegunMultiplayer {
                domain: Domain {
                    snake: Some(snake), ..
                },
                ..
            } => vec![Camera::MouthCentered, Camera::BoundariesCentered],
            _ => vec![Camera::BoundariesCentered],
        }
    }

    pub fn transform_pos(
        &self,
        pos: domain::Pos,
        snake: Option<&domain::Snake>,
        boundaries: &domain::Boundaries,
        px_scale: f64,
    ) -> TransformedPos {
        let pos = TransformedPos::from(pos) * px_scale;

        match self.camera {
            Camera::MouthCentered => {
                // center camera to the mouth
                //
                // position of the mouth after the same transformations as of 'pos'

                let adjusted_mouth = TransformedPos::from(
                    snake
                        .as_ref()
                        .expect("snake to exist for this camera to work")
                        .mouth(),
                ) * px_scale;

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
                let b = boundaries;
                let b_max = b.max;
                let b_min = b.min;
                let adjusted_boundaries_center = TransformedPos::new(
                    (b_min.x + b_max.x) as f64 / 2.0,
                    (b_min.y + b_max.y) as f64 / 2.0,
                ) * px_scale;
                // assert!(self.refs.is_canvas_fit(self.state));
                let cd = self.refs.canvas_dimensions() / 2.0;
                let to_center_x = cd.width - adjusted_boundaries_center.x;
                let to_center_y = cd.height - adjusted_boundaries_center.y;

                TransformedPos::new(pos.x + to_center_x, pos.y + to_center_y)
            }
        }
    }

    fn draw_snake(
        &self,
        r: &CanvasRenderer,
        snake: &domain::Snake,
        boundaries: &domain::Boundaries,
        // distinguish controlled snake from others, by drawing another cirle on head
        style: bool,
        px_scale: f64,
    ) {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        let snake_body_width = SNAKE_BODY_WIDTH * px_scale;

        let transform_pos = |pos| self.transform_pos(pos, Some(snake), boundaries, px_scale);

        r.set_line_width(snake_body_width);
        let pos = transform_pos(snake.iter_vertices().next().unwrap());
        r.begin_path();
        r.move_to(pos);
        for pos in snake.iter_vertices().skip(1).map(transform_pos) {
            r.line_to(pos);
        }
        r.stroke();
        r.close_path();

        // round head
        let pos = transform_pos(snake.mouth());
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

        if style {
            let pos = transform_pos(snake.mouth());
            r.begin_path();
            r.cirle(pos, (snake_body_width / 2.) * 0.3);
            r.set_fill_style(box_border_color);
            r.fill();
            r.close_path();
        }

        // round tail
        let pos = transform_pos(snake.tail_end());
        r.begin_path();
        r.cirle(pos, snake_body_width / 2.);
        r.set_fill_style(box_border_color);
        r.fill();
        r.close_path();

        // if let Some(label) = label {
        //     let pos = transform_pos(snake.mouth());
        //     r.begin_path();
        //     r.set_text_align("center");
        //     r.set_font("30px serif");
        //     r.fill_text(label, pos);
        //     r.close_path();
        // }
    }

    fn draw_foods(
        &self,
        r: &CanvasRenderer,
        foods: &domain::Foods,
        snake: Option<&domain::Snake>,
        boundaries: &domain::Boundaries,
        px_scale: f64,
    ) {
        let theme = self.theme_ctx.as_ref();
        let bg_color = &theme.bg_color;
        let box_border_color = &theme.box_border_color;

        for food in foods.iter() {
            let pos = self.transform_pos(food.pos, snake, boundaries, px_scale);
            r.begin_path();
            r.cirle(pos, FOOD_DIAMETER * px_scale / 2.);
            r.set_fill_style(box_border_color);
            r.fill();
            r.close_path();
        }
    }

    fn draw_boundaries(
        &self,
        r: &CanvasRenderer,
        boundaries: &domain::Boundaries,
        snake: Option<&domain::Snake>,
        px_scale: f64,
    ) {
        let transform_pos = |pos| self.transform_pos(pos, snake, boundaries, px_scale);

        r.set_line_width(0.5 * px_scale);
        let pos = transform_pos(boundaries.left_top());

        r.begin_path();
        r.move_to(pos);

        for pos in [
            boundaries.right_top(),
            boundaries.right_bottom(),
            boundaries.left_bottom(),
            boundaries.left_top(),
        ] {
            let pos = transform_pos(pos);
            r.line_to(pos);
        }
        r.close_path();
        r.stroke();
    }
}

#[derive(Clone, PartialEq)]
pub enum MPLobbyState {
    ToJoin {
        lobby_name: LobbyName,
    },
    JoinError {
        lobby_name: LobbyName,
        message: String,
    },
    Joined,
}

#[derive(Clone, PartialEq)]
pub enum NotBegunState {
    ModeSelection,
    MPCreateJoinLobby,
    MPCreateLobby,
    MPLobby {
        state: MPLobbyState,
    },
    MPSetUsername {
        next_state: Box<NotBegunState>,
    },
    MPLobbyList {
        lobbies: Option<interfacing::snake::LobbyList>,
    },
    Initial,
    Ended,
}

// #[derive(Clone, PartialEq)]
// struct BegunSinglePlayerState {
//     paused: bool,
// }

// #[derive(Clone, PartialEq)]
// struct BegunMultiPlayerState;

// #[derive(Clone, PartialEq)]
// enum BegunState {
//     Singleplayer { paused: bool },
//     Multiplayer,
// }

pub enum State {
    BegunSingleplayer {
        snake: domain::Snake,
        foods: domain::Foods,
        boundaries: domain::Boundaries,
        // greater value - closer camera
        px_scale: f64,

        advance_interval: SnakeAdvanceInterval,
    },
    BegunMultiplayer {
        domain: Domain,
        // greater value - closer camera
        px_scale: f64,
    },
    NotBegun {
        inner: NotBegunState,
    },
}

impl State {
    pub fn to_be_loaded_lobby(lobby_name: LobbyName) -> Self {
        State::NotBegun {
            inner: NotBegunState::MPLobby {
                state: MPLobbyState::ToJoin { lobby_name },
            },
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Camera {
    MouthCentered,
    BoundariesCentered,
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
            State::NotBegun { .. } => true,
            State::BegunSingleplayer { .. } | State::BegunMultiplayer { .. } => {
                self.canvas_dimensions() == canvas_target_dimensions()
            }
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
    location_listener: LocationHandle,
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

        let location_listener = link
            .add_location_listener(link.callback(|e: Location| {
                let path = e.path();

                use crate::router::Route;

                let route = Route::recognize(path);

                // console::log!("!!! location changed", path, format!("{route:?}"));

                if let Some(route) = route.clone() {
                    let state = match route {
                        Route::Home | Route::Snake => Some(NotBegunState::ModeSelection),
                        Route::SnakeLobby { lobby_name } => Some(NotBegunState::MPLobby {
                            state: MPLobbyState::ToJoin { lobby_name },
                        }),
                        Route::SnakeCreateJoinLobby => Some(NotBegunState::MPCreateJoinLobby),
                        Route::SnakeCreateLobby => Some(NotBegunState::MPCreateLobby),
                        Route::SnakeLobbies => Some(NotBegunState::MPLobbyList { lobbies: None }),
                        _ => None,
                    };

                    if let Some(state) = state {
                        return SnakeMsg::StateChange(State::NotBegun { inner: state });
                    }
                }

                SnakeMsg::Nothing
            }))
            .unwrap();

        Self {
            kb_listener,
            window_load_listener,
            window_resize_listener,
            location_listener,
        }
    }
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

        let mut foods = domain::Foods::default();
        foods.extend(values.into_iter().map(domain::Food::from));
        foods
    }

    fn boundaries(snake: &domain::Snake) -> domain::Boundaries {
        snake
            .mouth()
            .boundaries_in_radius(MAP_BOUNDARIES_X, MAP_BOUNDARIES_Y)
    }
}

fn default_domain() -> Domain {
    let snake = DomainDefaults::snake();
    let boundaries = DomainDefaults::boundaries(&snake);

    let food_average = ((MAP_BOUNDARIES_X * MAP_BOUNDARIES_Y) as f64 * 0.5);
    Domain {
        foods: DomainDefaults::foods(
            rand_from_iterator(((food_average * 0.9) as i32)..((food_average * 1.1) as i32)),
            boundaries,
            snake.iter_vertices(),
        ),
        other_snakes: Default::default(),
        boundaries,
        snake: Some(snake),
    }
}

struct SPDomainDefaults {
    snake: domain::Snake,
    foods: domain::Foods,
    boundaries: domain::Boundaries,
}

impl SPDomainDefaults {
    fn defaults() -> Self {
        let snake = DomainDefaults::snake();
        let boundaries = DomainDefaults::boundaries(&snake);

        let foods = {
            let food_average = ((MAP_BOUNDARIES_X * MAP_BOUNDARIES_Y) as f64 * 0.5);

            DomainDefaults::foods(
                rand_from_iterator(((food_average * 0.9) as i32)..((food_average * 1.1) as i32)),
                boundaries,
                snake.iter_vertices(),
            )
        };

        Self {
            snake,
            boundaries,
            foods,
        }
    }
}

pub struct SnakeAdvanceInterval {
    paused: bool,
    link: Scope<Snake>,
    _handle: Option<Interval>,
}

impl SnakeAdvanceInterval {
    fn create(link: Scope<Snake>) -> Self {
        Self {
            paused: true,
            link,
            _handle: None,
        }
    }

    fn paused(&self) -> bool {
        self.paused
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
        self.paused = false;
    }

    fn start(&mut self) {
        // don't reset if already started
        if let None = self._handle {
            self.reset();
        }
    }

    fn stop(&mut self) {
        self._handle = None;
        self.paused = true;
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

    fn stroke_text(&self, text: &str, TransformedPos { x, y }: TransformedPos) {
        self.as_ref().stroke_text(text, x, y);
    }

    fn fill_text(&self, text: &str, TransformedPos { x, y }: TransformedPos) {
        self.as_ref().fill_text(text, x, y);
    }

    fn set_font(&self, value: &str) {
        self.as_ref().set_font(value);
    }

    fn set_text_align(&self, value: &str) {
        self.as_ref().set_text_align(value);
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

fn calculate_px_scale(canvas_dimensions: Dimensions, boundaries: &domain::Boundaries) -> f64 {
    // scale game to fully fit into space available to canvas
    // when camera is Camera::BoundariesCentered

    // add ones to for bounds strokes to be inside space boundaries
    // can be chosen more accurate value in px
    f64::min(
        canvas_dimensions.height as f64 / (boundaries.height() + 1) as f64,
        canvas_dimensions.width as f64 / (boundaries.width() + 1) as f64,
    )
}

pub async fn sleep(delay: i32) {
    let mut cb = |resolve: js_sys::Function, reject: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay);
    };

    let p = js_sys::Promise::new(&mut cb);

    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

const UPDATE: bool = true;

impl Snake {
    fn handle_state_change(
        &mut self,
        ctx: &Context<Self>,
        s: interfacing::snake::LobbyState,
    ) -> bool {
        console::log!(format!("state change: {s:?}"));

        match &s {
            LobbyState::Prep(s) => {}
            LobbyState::Running(interfacing::snake::lobby_state::LobbyRunning {
                domain, ..
            }) => {
                ctx.link()
                    .send_message(SnakeMsg::StateChange(State::BegunMultiplayer {
                        domain: domain.clone(),
                        px_scale: calc_px_scale(&domain.boundaries),
                    }));
            }
            LobbyState::Terminated => {
                console::log!("should not receive");
            }
        }

        self.ws_state.joined_lobby_state.replace(s);

        UPDATE
    }

    fn handle_received_message(&mut self, ctx: &Context<Self>, msg: WsMsg<WsServerMsg>) -> bool {
        console::log!(format!("recv: {msg:?}"));

        match msg {
            WsMsg(Some(id), msg) => {
                let ack_msg = self.acknowledgeable_messages.get(&id);

                let ack_msg = if let None = ack_msg {
                    console::log!(format!("dissmiss response: {id} {msg:?}"));
                    return !UPDATE;
                } else {
                    ack_msg.unwrap()
                };

                // TODO decouple action to take somehow
                //
                match (ack_msg, msg) {
                    (WsClientMsg::LobbyList, WsServerMsg::LobbyList(lobby_list)) => {
                        if let State::NotBegun {
                            inner: NotBegunState::MPLobbyList { lobbies },
                        } = &mut self.state
                        {
                            lobbies.replace(lobby_list);
                            return UPDATE;
                        }
                    }

                    (WsClientMsg::UserName, WsServerMsg::UserName(user_name)) => {
                        console::log!(format!("setting ws_state.user_name {:?}", &user_name));
                        self.ws_state.user_name = user_name;
                        self.ws_state.synced_user_name = true;
                        return UPDATE;
                    }

                    (WsClientMsg::SetUserName(user_name), WsServerMsg::Ack) => {
                        console::log!("ack:", &id, format!("{ack_msg:?}"));
                        self.ws_state.user_name = Some(user_name.clone());
                        self.ws_state.synced_user_name = true;
                        return UPDATE;
                    }

                    (WsClientMsg::JoinLobby(lobby_name), WsServerMsg::LobbyState(s)) => {
                        console::log!("ack:", &id, format!("{ack_msg:?}"));

                        self.ws_state.joined_lobby_name = Some(lobby_name.clone());
                        self.ws_state.joined_lobby_state = Some(s);

                        ctx.link()
                            .send_message(SnakeMsg::StateChange(State::NotBegun {
                                inner: NotBegunState::MPLobby {
                                    state: MPLobbyState::Joined,
                                },
                            }));
                    }

                    (WsClientMsg::JoinLobby(ln), WsServerMsg::JoinLobbyDecline(r)) => {
                        console::log!("dec:", &id, format!("{ack_msg:?} {r:?}"));

                        // TODO handle self.ws_state.joined_lobby if needed

                        let s = match r {
                            JoinLobbyDecline::NotFound => {
                                console::log!("Lobby ", ln, " does not exist. Redirecting.");
                                NotBegunState::MPCreateJoinLobby
                            }
                            _ => NotBegunState::MPLobby {
                                state: MPLobbyState::JoinError {
                                    lobby_name: ln.clone(),
                                    message: format!("{r:?}"),
                                },
                            },
                        };

                        ctx.link()
                            .send_message(SnakeMsg::StateChange(State::NotBegun { inner: s }));
                    }

                    (WsClientMsg::VoteStart(_), WsServerMsg::LobbyState(s)) => {
                        return self.handle_state_change(ctx, s);
                    }

                    (WsClientMsg::LeaveLobby, WsServerMsg::Ack) => {
                        self.ws_state.joined_lobby_name = None;
                        self.ws_state.joined_lobby_state = None;

                        ctx.link()
                            .send_message(SnakeMsg::StateChange(State::NotBegun {
                                inner: NotBegunState::MPLobbyList { lobbies: None },
                            }));
                    }

                    (WsClientMsg::LeaveLobby, WsServerMsg::LeaveLobbyDecline(_)) => {
                        unreachable!("server should not send this message")
                    }

                    (req, res) => {
                        console::log!(format!("invalid response to request: {req:?} {res:?}"));
                        return UPDATE;
                    }
                }

                let msg = self.acknowledgeable_messages.remove(&id);
            }

            WsMsg(None, msg) => match msg {
                WsServerMsg::Ack => unreachable!("server should not send this message"),

                WsServerMsg::LobbyState(s) => {
                    return self.handle_state_change(ctx, s);
                }

                recv => console::log!(format!("invalid recv: {recv:?}")),
            },
        }

        !UPDATE
    }
}
