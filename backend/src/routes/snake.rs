use crate::mp_snake::{Lobbies, Lobby};
use crate::routes::imports::*;

// for debugging
const AUTO_GEN_USER_NAME: bool = true;

#[axum_macros::debug_handler]
pub async fn create_lobby(
    Extension(lobbies): Extension<Lobbies>,
    lobby: Json<interfacing::snake::CreateLobby>,
) -> ApiResult<impl IntoResponse> {
    tracing::info!("{:?}", lobby);

    let lobby = Lobby::new(lobby.name.clone());
    let result = lobbies.insert_if_missing(lobby).await;

    match result {
        Ok(()) => Ok(StatusCode::OK),
        Err(msg) => Err(ApiError::Conflict(msg)),
    }
}

#[axum_macros::debug_handler]
pub async fn get_lobby(
    Extension(lobbies): Extension<Lobbies>,
    Path(name): Path<interfacing::snake::LobbyName>,
) -> ApiResult<impl IntoResponse> {
    match lobbies.get(&name).await {
        None => Err(ApiError::EntryNotFound),
        Some(lobby) => {
            let name = lobby.read().await.name.clone();
            Ok(Json(interfacing::snake::GetLobby { name }))
        }
    }
}

pub mod ws {
    use crate::configuration::get_env;
    use crate::routes::imports::*;
    use crate::startup::UserConnectInfo;
    use axum::extract::connect_info::ConnectInfo;
    use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
    use futures_util::{
        sink::SinkExt,
        stream::{SplitSink, SplitStream, StreamExt},
    };
    use interfacing::snake::WsMsg;
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    use crate::mp_snake::{ws::State, Con, JoinLobbyError, Lobbies, PlayerUserNames};

    pub async fn ws(
        maybe_ws: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>,
        ConnectInfo(con_info): ConnectInfo<UserConnectInfo>,
        headers: hyper::HeaderMap,
        Extension(lobbies): Extension<Lobbies>,
        Extension(uns): Extension<PlayerUserNames>,
    ) -> Response {
        let ws = match maybe_ws {
            Ok(ws) => ws,
            Err(e) => {
                tracing::trace!("{headers:?}");
                tracing::error!("{e}");
                return e.into_response();
            }
        };

        let sock_addr = con_info.socket_addr(&headers);
        if get_env().local() {
            tracing::info!("Client connected to Snake Ws: {:?}", sock_addr);
        } else {
            tracing::info!("Client connected to Snake Ws");
        }

        // connection identifier,
        // expected to be unique across current state
        let con = sock_addr.port();

        ws.on_upgrade(move |socket| handle_socket(socket, con, lobbies, uns))
    }

    type ClientMsg = WsMsg<interfacing::snake::WsClientMsg>;
    type ServerMsg = WsMsg<interfacing::snake::WsServerMsg>;

    async fn handle_socket(socket: WebSocket, con: Con, lobbies: Lobbies, uns: PlayerUserNames) {
        let con_state = {
            let mut state = State::default();

            state.user_name = if super::AUTO_GEN_USER_NAME {
                let un = format!("Player {con}");
                // do not handle possible collision, since it's debug only feature
                uns.try_insert(un.clone(), con).await.unwrap();
                Some(un)
            } else {
                None
            };

            Arc::new(Mutex::new(state))
        };

        let (server_msg_sender, server_msg_receiver) = mpsc::unbounded_channel::<ServerMsg>();

        let (sender, receiver) = socket.split();
        let rh = tokio::spawn(read(
            receiver,
            con_state.clone(),
            server_msg_sender.clone(),
            lobbies.clone(),
            con.clone(),
            uns.clone(),
        ));
        let wh = tokio::spawn(write(sender, server_msg_receiver));

        // as soon as a closed channel error returns from any of these procedures,
        // cancel the other
        () = tokio::select! {
            _ = rh => (),
            _ = wh => (),
        };

        // TODO investigate when port becomes free
        // undefined behavior is possible, if port can become free before this sections running
        //
        // clean up
        lobbies.disjoin_con(con).await;
        uns.clean_con(con).await;
    }

    async fn read(
        mut receiver: SplitStream<WebSocket>,
        con_state: Arc<Mutex<State>>,
        server_msg_sender: mpsc::UnboundedSender<ServerMsg>,
        lobbies: Lobbies,
        con: Con,
        uns: PlayerUserNames,
    ) {
        loop {
            match receiver.next().await {
                Some(Ok(Message::Text(text))) => match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(msg) => {
                        tracing::info!("Received message: {text:?}");
                        tokio::task::spawn(handle_received_message(
                            msg,
                            con_state.clone(),
                            server_msg_sender.clone(),
                            lobbies.clone(),
                            con.clone(),
                            uns.clone(),
                        ));
                    }
                    Err(_) => {
                        tracing::info!("Received unexpected message: {text:?}");
                    }
                },
                Some(Ok(msg)) => {
                    tracing::info!("Received unhandled message: {msg:?}");
                }
                Some(Err(_)) => {
                    tracing::info!("Client disconnected");
                    return;
                }
                None => {
                    tracing::info!("Broadcast channel closed");
                    return;
                }
            }
        }
    }

    async fn handle_received_message(
        msg: ClientMsg,
        con_state: Arc<Mutex<State>>,
        server_msg_sender: mpsc::UnboundedSender<ServerMsg>,
        lobbies: Lobbies,
        con: Con,
        uns: PlayerUserNames,
    ) {
        use interfacing::snake::{PinnedMessage, WsClientMsg::*, WsServerMsg};

        let con = con;

        match msg {
            WsMsg(Some(id), SetUserName(value)) => {
                let send = if lobbies.joined_any(con).await {
                    // forbid name changing when joined lobby
                    interfacing::snake::WsServerMsg::ForbiddenWhenJoined
                } else {
                    match uns.try_insert(value.clone(), con).await {
                        Ok(()) => {
                            con_state.lock().await.user_name.replace(value);
                            WsServerMsg::Ack
                        }
                        Err(()) => interfacing::snake::WsServerMsg::UserNameOccupied,
                    }
                };
                server_msg_sender.send(id.pinned_msg(send)).unwrap();
            }

            WsMsg(Some(id), UserName) => {
                let user_name = con_state.lock().await.user_name.clone();
                let send = interfacing::snake::WsServerMsg::UserName(user_name);
                server_msg_sender.send(id.pinned_msg(send)).unwrap();
            }

            WsMsg(Some(id), JoinLobby(lobby_name)) => {
                use interfacing::snake::JoinLobbyDecline;

                let send = match &con_state.lock().await.user_name {
                    None => interfacing::snake::WsServerMsg::JoinLobbyDecline(
                        interfacing::snake::JoinLobbyDecline::UserNameNotSet,
                    ),
                    Some(un) => {
                        match lobbies
                            .join_con(lobby_name, con, server_msg_sender.clone(), un.clone())
                            .await
                        {
                            Ok(()) => WsServerMsg::Ack,

                            Err(e) => {
                                // TODO impl From
                                let e = match e {
                                    JoinLobbyError::NotFound => JoinLobbyDecline::NotFound,
                                    JoinLobbyError::AlreadyJoined(lobby_name) => {
                                        JoinLobbyDecline::AlreadyJoined(lobby_name)
                                    }
                                    JoinLobbyError::AlreadyStarted => {
                                        JoinLobbyDecline::AlreadyStarted
                                    }
                                };
                                WsServerMsg::JoinLobbyDecline(e)
                            }
                        }
                    }
                };

                server_msg_sender.send(id.pinned_msg(send)).unwrap();
            }

            WsMsg(Some(id), LobbyList) => {
                let lobby_list = lobbies
                    .read()
                    .await
                    .keys()
                    .map(|lobby_name| interfacing::snake::list::Lobby {
                        name: lobby_name.clone(),
                    })
                    .collect::<Vec<_>>();

                let send = WsServerMsg::LobbyList(lobby_list);
                server_msg_sender.send(id.pinned_msg(send)).unwrap();
            }

            WsMsg(Some(id), VoteStart(value)) => {
                let lobby = lobbies.joined_lobby(con).await;

                match lobby {
                    None => {
                        let send = WsServerMsg::Err("lobby does not exist".into());
                        server_msg_sender.send(id.pinned_msg(send)).unwrap();
                    }
                    Some(lobby) => {
                        let mut lock = lobby.write().await;
                        let result = lock.vote_start(con, value);

                        match result {
                            Ok(()) => {
                                lock.pinned_broadcast_state(id, con);
                            }
                            Err(m) => {
                                // this branch handling is required because
                                // it's possible that between lobbies.joined_lobby and lobby.vote_start
                                // player leaves the lobby
                                let send = WsServerMsg::Err(m);
                                server_msg_sender.send(id.pinned_msg(send)).unwrap();
                            }
                        };
                    }
                };
            }

            WsMsg(None, JoinLobby(_) | UserName | LobbyList | SetUserName(_) | VoteStart(_)) => {
                // TODO do not panic in prod
                unreachable!("ack expected")
            }
        }
    }

    async fn write(
        mut sender: SplitSink<WebSocket, Message>,
        mut server_msg_receiver: mpsc::UnboundedReceiver<ServerMsg>,
    ) {
        while let Some(msg) = server_msg_receiver.recv().await {
            let msg = Message::Text(serde_json::to_string(&msg).unwrap());

            match sender.send(msg.clone()).await {
                Ok(()) => {
                    tracing::info!("Sent message: {msg:?}")
                }
                Err(_) => {
                    tracing::info!("Client disconnected");
                    return;
                }
            }
        }
    }
}
