use crate::mp_snake;
use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn create_lobby(
    Extension(lobbies): Extension<mp_snake::Lobbies>,
    lobby: Json<interfacing::snake::CreateLobby>,
) -> ApiResult<impl IntoResponse> {
    tracing::info!("{:?}", lobby);

    let lobby = mp_snake::Lobby::new(lobby.name.clone());
    let result = lobbies.insert_if_missing(lobby).await;

    match result {
        Ok(()) => Ok(StatusCode::OK),
        Err(msg) => Err(ApiError::Conflict(msg)),
    }
}

#[axum_macros::debug_handler]
pub async fn get_lobby(
    Extension(lobbies): Extension<mp_snake::Lobbies>,
    Path(name): Path<interfacing::snake::LobbyName>,
) -> ApiResult<impl IntoResponse> {
    let result = lobbies
        .get(&name)
        .await
        .map(|lobby| interfacing::snake::GetLobby { name: lobby.name });

    match result {
        Some(lobby) => Ok(Json(lobby)),
        None => Err(ApiError::EntryNotFound),
    }
}

pub mod ws {
    use crate::configuration::get_env;
    use crate::mp_snake;
    use crate::routes::imports::*;
    use crate::startup::UserConnectInfo;
    use axum::extract::connect_info::ConnectInfo;
    use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
    use futures_util::{
        sink::SinkExt,
        stream::{SplitSink, SplitStream, StreamExt},
    };
    use interfacing::snake::WsMsg;
    use mp_snake::ws::State;
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    pub async fn ws(
        maybe_ws: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>,
        ConnectInfo(con_info): ConnectInfo<UserConnectInfo>,
        headers: hyper::HeaderMap,
        Extension(lobbies): Extension<mp_snake::Lobbies>,
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

        ws.on_upgrade(move |socket| handle_socket(socket, sock_addr, lobbies))
    }

    type ClientMsg = WsMsg<interfacing::snake::WsClientMsg>;
    type ServerMsg = WsMsg<interfacing::snake::WsServerMsg>;

    async fn handle_socket(
        socket: WebSocket,
        sock: std::net::SocketAddr,
        #[allow(unused)] lobbies: mp_snake::Lobbies,
    ) {
        if get_env().local() {
            tracing::info!("Client connected to Snake Ws: {:?}", sock);
        } else {
            tracing::info!("Client connected to Snake Ws");
        }

        let con_state = Arc::new(Mutex::new(State::default()));

        let (server_msg_sender, server_msg_receiver) = mpsc::unbounded_channel::<ServerMsg>();

        let (sender, receiver) = socket.split();
        let rh = tokio::spawn(read(receiver, con_state.clone(), server_msg_sender.clone()));
        let wh = tokio::spawn(write(sender, server_msg_receiver));

        // as soon as a closed channel error returns from any of these procedures,
        // cancel the other
        let () = tokio::select! {
            _ = rh => (),
            _ = wh => (),
        };
    }

    async fn read(
        mut receiver: SplitStream<WebSocket>,
        con_state: Arc<Mutex<State>>,
        server_msg_sender: mpsc::UnboundedSender<ServerMsg>,
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
                                WsMsg(_, SetUserName(value)) => {
                                    {
                                        con_state.lock().await.user_name.replace(value);
                                    }
                                    {
                                        if let Some(ack) = ack {
                                            server_msg_sender.send(ack).unwrap();
                                        }
                                    }
                                }
                                WsMsg(id, UserName) => {
                                    let user_name = con_state.lock().await.user_name.clone();
                                    server_msg_sender
                                        .send(
                                            WsMsg::new(interfacing::snake::WsServerMsg::UserName(
                                                user_name,
                                            ))
                                            .maybe_id(id),
                                        )
                                        .unwrap();
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
