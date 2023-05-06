use crate::registry::{NodeMsg, RegistryMsg};
use crate::proto::WSProto;
use crate::state::Handles;
use crate::Registry;
use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};

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
    let mut handle = registry.register().await;
    let mut cycles = 0;

    loop {
        match handle.receiver.try_recv() {
            Ok(NodeMsg::Invoke(name, args)) => {
                println!("Worker::Processing");
                let ws_msg = WSProto::invoke_request(name, args);

                socket.send(Message::Text(ws_msg)).await.unwrap();
                //process_msg(registry.clone(), &mut socket, &handle, msg).await;
                let response = socket.recv().await.unwrap();
                match response {
                    Ok(response_txt) => {
                        println!("Worker responded: {:?}", response_txt);
                        let body = response_txt.into_text().unwrap();
                        //let val: serde_json::Value = serde_json::from_str(&body).unwrap();
                        //let response: WSProto = serde_json::from_value(val).unwrap();
                        let msg = RegistryMsg::InvokeResult(body);
                        handle.sender.send(msg).await.unwrap();
                    }
                    _ => println!("ERROR FROM RESPONSE"),
                }
                println!("Processde");
                continue;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                cycles = (cycles + 1) % 1000;
                if cycles == 0 {
                    let ping = socket.send(Message::Ping(vec![41, 41, 41])).await;
                    if ping.is_err() {
                        println!("Disconnecting..");
                        registry.disconnect(handle).await;
                        return
                    } else {
                        socket.recv().await.unwrap().unwrap();
                    }
                }
            }
        }
    }
    println!("Disconnected");
}

struct ClientError {}
//async fn process_msg(registry: Arc<Registry>, sock:&mut WebSocket, handle: &BackendHandle, msg: BEProtocol) {
//    match msg {
//        BEProtocol::Invoke(name) => {
//            println!("Request from registry to invoke: {}", name);
//            sock.send(Message::Text(name)).await.unwrap();
//            match sock.recv().await.unwrap() {
//                Ok(msg) => {
//                    println!("Worker replied: {:?}", msg);
//                    handle.send(WorkerProtocol::InvokeResult(msg.into_text().unwrap())).unwrap();
//                },
//                _ => {
//                    println!("Disconnecting 2");
//                    registry.disconnect(handle);
//                }
//            }
//        }
//        _ => panic!("Unknown msg")
//    }
//}
