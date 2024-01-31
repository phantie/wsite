use crate::routes::imports::*;

use crate::startup::mp_snake;

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
    Path(name): Path<mp_snake::LobbyName>,
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
    use crate::routes::imports::*;
    use crate::startup::mp_snake;
    use crate::startup::UserConnectInfo;
    use axum::extract::connect_info::ConnectInfo;
    use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
    use futures_util::{
        sink::SinkExt,
        stream::{SplitSink, SplitStream, StreamExt},
    };
    use mp_snake::ws::State;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub async fn ws(
        maybe_ws: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>,
        ConnectInfo(con_info): ConnectInfo<UserConnectInfo>,
        headers: hyper::HeaderMap,
        Extension(lobbies): Extension<mp_snake::Lobbies>,
        Extension(users_online): Extension<UsersOnline>,
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

        ws.on_upgrade(move |socket| handle_socket(socket, users_online, sock_addr, lobbies))
    }

    async fn handle_socket(
        socket: WebSocket,
        users_online: UsersOnline,
        sock: std::net::SocketAddr,
        #[allow(unused)] lobbies: mp_snake::Lobbies,
    ) {
        if get_env().local() {
            tracing::info!("Client connected to Snake Ws: {:?}", sock);
        } else {
            tracing::info!("Client connected to Snake Ws");
        }

        let con_state = Arc::new(Mutex::new(State::default()));

        let (sender, receiver) = socket.split();
        let rh = tokio::spawn(read(receiver, con_state.clone()));
        let con_count_r = users_online.con_count_r.clone();
        let wh = tokio::spawn(write(sender, con_count_r, con_state.clone()));

        // as soon as a closed channel error returns from any of these procedures,
        // cancel the other
        let () = tokio::select! {
            _ = rh => (),
            _ = wh => (),
        };
    }

    async fn read(mut receiver: SplitStream<WebSocket>, con_state: Arc<Mutex<State>>) {
        loop {
            match receiver.next().await {
                Some(Ok(msg)) => {
                    match &msg {
                        Message::Text(msg) => {
                            use interfacing::snake::Msg;
                            use interfacing::snake::WsClientMsg::*;

                            let msg = serde_json::from_str::<
                                /* TODO name this type in there */
                                interfacing::snake::Msg<interfacing::snake::WsClientMsg>,
                            >(msg)
                            .unwrap(); // TODO handle

                            match msg {
                                Msg(_, UserName(value)) => {
                                    con_state.lock().await.user_name.replace(value);
                                    unimplemented!()
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
        mut con_count_r: async_broadcast::Receiver<usize>,
        #[allow(unused)] con_state: Arc<Mutex<State>>,
    ) {
        loop {
            match con_count_r.recv().await {
                Ok(i) => {
                    let msg = Message::Text(format!("users_online:{i}"));
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
                Err(async_broadcast::RecvError::Overflowed(_)) => {}
                Err(async_broadcast::RecvError::Closed) => return,
            }
        }
    }
}
