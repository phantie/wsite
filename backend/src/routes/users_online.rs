#![allow(unused)]
use crate::routes::imports::*;
use crate::startup::UserConnectInfo;
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

async fn handle_socket(socket: WebSocket, state: AppState, connect_info: UserConnectInfo) {
    tracing::info!("Client connected: {:?}", connect_info.remote_addr);
    {
        let ips = &mut state.users_online.ips.write().await;
        let pages_per_ip = *ips.entry(connect_info.remote_addr).or_default() + 1;
        ips.insert(connect_info.remote_addr, pages_per_ip);

        let ip_count = ips.len() as i32;
        state
            .users_online
            .count_s
            .broadcast(ip_count)
            .await
            .unwrap();
        tracing::info!("Broadcasted user count after add: {ip_count} {ips:?}");
    }

    let (sender, receiver) = socket.split();
    let rh = tokio::spawn(read(receiver));
    let count_r = state.users_online.count_r.clone();
    let wh = tokio::spawn(write(sender, count_r));

    // as soon as a closed channel error returns from any of these procedures,
    // cancel the other
    let () = tokio::select! {
        _ = rh => (),
        _ = wh => (),
    };

    {
        let ips = &mut state.users_online.ips.write().await;
        let count = *ips
            .get(&connect_info.remote_addr)
            .expect("Connect before disconnect");
        if count == 1 {
            ips.remove(&connect_info.remote_addr);
        } else {
            ips.insert(connect_info.remote_addr, count - 1);
        }

        let ip_count = ips.len() as i32;
        state
            .users_online
            .count_s
            .broadcast(ip_count)
            .await
            .unwrap();
        tracing::info!("Broadcasted user count after delete: {ip_count} {ips:?}");
    }
}

async fn read(mut receiver: SplitStream<WebSocket>) {
    loop {
        match receiver.next().await {
            Some(item) => match item {
                Ok(msg) => {
                    tracing::info!("Received message: {msg:?}");
                }
                Err(_) => {
                    tracing::info!("Client disconnected");
                    return;
                }
            },
            None => {
                tracing::info!("Broadcast channel closed");
                return;
            }
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut count_r: async_broadcast::Receiver<i32>,
) {
    loop {
        match count_r.recv().await {
            Ok(i) => {
                let msg = Message::Text(format!("users_online:{i}"));
                // TODO fix first message sent to client is skipped
                for _ in 0..=1 {
                    match sender.feed(msg.clone()).await {
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
            Err(async_broadcast::RecvError::Overflowed(_)) => {}
            Err(async_broadcast::RecvError::Closed) => return,
        }
    }
}
