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
    // ws: WebSocketUpgrade,
    maybe_ws: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>,
    ConnectInfo(con_info): ConnectInfo<UserConnectInfo>,
    headers: hyper::HeaderMap,
    State(state): State<AppState>,
) -> Response {
    tracing::info!("{:?}", headers);
    let ws = match maybe_ws {
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!("{}", &e);
            return e.into_response();
        }
    };

    ws.on_upgrade(|socket| handle_socket(socket, state, con_info))
}

async fn handle_socket(socket: WebSocket, state: AppState, con_info: UserConnectInfo) {
    tracing::info!("Client connected: {:?}", con_info.remote_addr);

    {
        let cons = &mut state.users_online.cons.lock().await;
        let cons_per_ip = *cons.entry(con_info.remote_addr).or_default();
        cons.insert(con_info.remote_addr, cons_per_ip + 1);

        let con_count = cons.len();
        state.users_online.broadcast_con_count(con_count).await;
        tracing::info!("Broadcasted con count after add: {con_count}");
    }

    let (sender, receiver) = socket.split();
    let rh = tokio::spawn(read(receiver));
    let con_count_r = state.users_online.con_count_r.clone();
    let wh = tokio::spawn(write(sender, con_count_r));

    // as soon as a closed channel error returns from any of these procedures,
    // cancel the other
    let () = tokio::select! {
        _ = rh => (),
        _ = wh => (),
    };

    {
        let cons = &mut state.users_online.cons.lock().await;
        let cons_per_ip = *cons
            .get(&con_info.remote_addr)
            .expect("Connect predates disconnect");
        if cons_per_ip == 1 {
            cons.remove(&con_info.remote_addr);
        } else {
            cons.insert(con_info.remote_addr, cons_per_ip - 1);
        }

        let con_count = cons.len();
        state.users_online.broadcast_con_count(con_count).await;
        tracing::info!("Broadcasted con count after delete: {con_count}");
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
    mut con_count_r: async_broadcast::Receiver<usize>,
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
