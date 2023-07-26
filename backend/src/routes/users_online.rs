#![allow(unused)]
use crate::routes::imports::*;
use crate::startup::{SharedUsersOnline, UserConnectInfo};
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

#[axum_macros::debug_handler]
pub async fn ws_users_online(
    ws: WebSocketUpgrade,
    ConnectInfo(connect_info): ConnectInfo<UserConnectInfo>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, connect_info))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, connect_info: UserConnectInfo) {
    tracing::info!("Client connected");
    {
        let mut ips = &mut state.users_online.write().await.ips;
        let count = *ips.entry(connect_info.remote_addr).or_default() + 1;
        ips.insert(connect_info.remote_addr, count);
    }

    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(read(receiver));
    tokio::spawn(write(sender, state, connect_info));
}

async fn read(mut receiver: SplitStream<WebSocket>) {
    loop {
        match receiver.next().await {
            Some(item) => match item {
                Ok(msg) => {
                    tracing::info!("received message: {msg:?}");
                }
                Err(_) => {
                    tracing::info!("Client disconnected");
                    return;
                }
            },
            None => {
                tracing::info!("Connection closed by writer");
                return;
            }
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    state: AppState,
    connect_info: UserConnectInfo,
) {
    let msg = if crate::configuration::get_env().local() {
        Message::Text(format!(
            "users_online:{:?}",
            state.users_online.read().await.ips
        ))
    } else {
        unimplemented!()
    };

    loop {
        match sender.feed(msg.clone()).await {
            Ok(()) => {
                tracing::info!("sent message: {msg:?}")
            }
            Err(_) => {
                tracing::info!("Client disconnected");
                {
                    let mut ips = &mut state.users_online.write().await.ips;
                    let count = *ips
                        .get(&connect_info.remote_addr)
                        .expect("connect before disconnect");
                    if count == 1 {
                        ips.remove(&connect_info.remote_addr);
                    } else {
                        ips.insert(connect_info.remote_addr, count - 1);
                    }
                }

                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
