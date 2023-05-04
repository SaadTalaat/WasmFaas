use crate::state::Handles;
use crate::Registry;
use crate::registry::{BEProtocol, BackendHandle, WorkerProtocol};
use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, path::PathBuf};
use std::sync::Arc;

pub async fn ws_handler(
    Extension(handles): Extension<Handles>,
    upgrade: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let registry = handles.registry;
    upgrade.on_upgrade(move |socket| handle(registry, socket, addr))
}

async fn handle(registry: Arc<Registry>, mut socket: WebSocket, addr: SocketAddr) {
    println!("Established: {}", addr);
    let handle = registry.register();
    loop {
        match handle.try_recv() {
            Ok(BEProtocol::WorkerDie) => {
                println!("Disconnecting");
                registry.disconnect(&handle);
                return;
            }
            Ok(msg) => {
                process_msg(registry.clone(), &mut socket, &handle, msg).await;
                println!("Processde");
                continue
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(3)).await;

            }
        }
    }
    println!("Disconnected");
}

struct ClientError {

}
async fn process_msg(registry: Arc<Registry>, sock:&mut WebSocket, handle: &BackendHandle, msg: BEProtocol) {
    match msg {
        BEProtocol::Invoke(name) => {
            println!("Request from registry to invoke: {}", name);
            sock.send(Message::Text(name)).await.unwrap();
            match sock.recv().await.unwrap() {
                Ok(msg) => {
                    println!("Worker replied: {:?}", msg);
                    handle.send(WorkerProtocol::InvokeResult(msg.into_text().unwrap())).unwrap();
                },
                _ => {
                    println!("Disconnecting 2");
                    registry.disconnect(handle);
                }
            }
        }
        _ => panic!("Unknown msg")
    }
}
