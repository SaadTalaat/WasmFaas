use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::{
        ws::{Message,WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf};

pub async fn ws_handler(
    upgrade: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| handle(socket,addr))
}

async fn handle(mut socket: WebSocket, addr: SocketAddr) {
    println!("Established: {}", addr);
    while true {
        if socket.send(Message::Text(String::from("tst"))).await.is_err() {
            println!("Connection disrupted");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
