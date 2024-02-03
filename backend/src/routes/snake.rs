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
            let Lobby { name, .. } = lobby.read().await.to_owned();
            Ok(Json(interfacing::snake::GetLobby { name }))
        }
    }
}

pub mod ws {
    use crate::configuration::get_env;
    use crate::mp_snake::{ws::State, JoinLobbyError, Lobbies, PlayerUserNames, Player};
    use crate::routes::imports::*;
    use crate::startup::UserConnectInfo;
    use axum::extract::connect_info::ConnectInfo;
    use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
    use futures_util::{
        sink::SinkExt,
        stream::{SplitSink, SplitStream, StreamExt},
    };
    use interfacing::snake::WsMsg;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    use super::AUTO_GEN_USER_NAME;

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
                tracing::trace!("{:?}", headers);
                tracing::error!("{}", &e);
                return e.into_response();
            }
        };

        let sock_addr = con_info.socket_addr(&headers);

        ws.on_upgrade(move |socket| handle_socket(socket, sock_addr, lobbies, uns))
    }

    type ClientMsg = WsMsg<interfacing::snake::WsClientMsg>;
    type ServerMsg = WsMsg<interfacing::snake::WsServerMsg>;

    async fn handle_socket(socket: WebSocket, sock_addr: SocketAddr, lobbies: Lobbies, uns: PlayerUserNames) {
        if get_env().local() {
            tracing::info!("Client connected to Snake Ws: {:?}", sock_addr);
        } else {
            tracing::info!("Client connected to Snake Ws");
        }

        let con_state = {
            let mut state = State::default();

            state.user_name = if AUTO_GEN_USER_NAME {
                let un = uuid::Uuid::new_v4().to_string();
                // do not handle possible collision, since it's debug only feature
                uns.try_insert(un.clone(), sock_addr).await.unwrap(); 
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
            sock_addr.clone(),
            uns.clone(),
        ));
        let wh = tokio::spawn(write(sender, server_msg_receiver));

        // as soon as a closed channel error returns from any of these procedures,
        // cancel the other
        let () = tokio::select! {
            _ = rh => (),
            _ = wh => (),
        };

        let player = Player { id: sock_addr.clone() };

        // clean up
        lobbies.disjoin_player(player).await;
        uns.clean_con(sock_addr).await;
    }

    async fn read(
        mut receiver: SplitStream<WebSocket>,
        con_state: Arc<Mutex<State>>,
        server_msg_sender: mpsc::UnboundedSender<ServerMsg>,
        lobbies: Lobbies,
        sock_addr: SocketAddr,
        uns: PlayerUserNames
    ) {
        loop {
            match receiver.next().await {
                Some(Ok(msg)) => {
                    match &msg {
                        Message::Text(msg) => {
                            use interfacing::snake::WsClientMsg::*;

                            let msg = serde_json::from_str::<ClientMsg>(msg).unwrap(); // TODO handle
                            let ack = msg.ack();

                            match msg {
                                WsMsg(id, SetUserName(value)) => {
                                    match uns.try_insert(value.clone(), sock_addr).await {
                                        Ok(()) => {
                                            con_state.lock().await.user_name.replace(value);

                                            if let Some(ack) = ack {
                                                server_msg_sender.send(ack).unwrap();
                                            }
                                        }
                                        Err(()) => {
                                            let msg = WsMsg::new(interfacing::snake::WsServerMsg::UserNameOccupied).maybe_id(id);
                                            server_msg_sender.send(msg).unwrap();
                                        }
                                    }
                                }

                                WsMsg(id, UserName) => {
                                    let user_name = con_state.lock().await.user_name.clone();
                                    let send = WsMsg::new(
                                        interfacing::snake::WsServerMsg::UserName(user_name),
                                    )
                                    .maybe_id(id);
                                    server_msg_sender.send(send).unwrap();
                                }

                                WsMsg(None, JoinLobby(_lobby_name)) => {
                                    unreachable!("ack required")
                                }

                                WsMsg(Some(id), JoinLobby(lobby_name)) => {
                                    use interfacing::snake::{JoinLobbyDecline, WsServerMsg};

                                    let send = match con_state.lock().await.user_name {
                                        None => {
                                            WsMsg::new(
                                                interfacing::snake::WsServerMsg::JoinLobbyDecline(interfacing::snake::JoinLobbyDecline::UserNameNotSet),
                                            )
                                            .id(id)
                                        }
                                        Some(_) => {
 
                                            let player = Player { id: sock_addr };

                                            match lobbies.join_player(lobby_name, player).await
                                            {
                                                Ok(()) => ack.unwrap(),
                                                Err(JoinLobbyError::NotFound) => WsMsg::new(WsServerMsg::JoinLobbyDecline(JoinLobbyDecline::NotFound)),
                                                Err(JoinLobbyError::AlreadyJoined(lobby_name)) => WsMsg::new(WsServerMsg::JoinLobbyDecline(JoinLobbyDecline::AlreadyJoined(lobby_name))),
                                            }
                                        }
                                    };

                                    server_msg_sender.send(send).unwrap();
                                }

                                WsMsg(id, LobbyList) => {
                                    use interfacing::snake::WsServerMsg;

                                    let lobby_list = lobbies.read().await.keys().map(|lobby_name| interfacing::snake::list::Lobby {
                                        name: lobby_name.clone()
                                    }).collect::<Vec<_>>();

                                    let send = WsMsg::new(WsServerMsg::LobbyList(lobby_list)).maybe_id(id);

                                    server_msg_sender.send(send).unwrap();
                                }
                            }
                        }
                        _ => {}
                    }

                    tracing::info!("Received message: {msg:?}");
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
