use crate::{
    extensions::Handles,
    proto::{NodeMsg, RegistryMsg, WSProto},
    registry::Registry,
    registry::RegistryHandle,
};
use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use std::{collections::HashMap, net::SocketAddr, ops::ControlFlow, sync::Arc};
use tokio::{
    sync::{oneshot::Sender, Mutex},
    time::Duration,
};
use uuid::Uuid;

/// Main purpose is to store oneshot channel
/// senders that are used to reply back to registry.
/// This pool is shared between both the worker_relay
/// task and the registry_relay task.
struct WSReplyPool {
    senders: Mutex<HashMap<String, Sender<RegistryMsg>>>,
}

impl WSReplyPool {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(
        &self,
        id: String,
        sender: Sender<RegistryMsg>,
    ) -> Option<Sender<RegistryMsg>> {
        self.senders.lock().await.insert(id, sender)
    }

    pub async fn reply(&self, id: &str, reply: RegistryMsg) -> Result<(), RegistryMsg> {
        let maybe_sender = self.senders.lock().await.remove(id);
        match maybe_sender {
            Some(sender) => sender.send(reply),
            None => Err(reply),
        }
    }
}

pub async fn ws_handler(
    Extension(handles): Extension<Handles>,
    upgrade: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let registry = handles.registry;
    upgrade.on_upgrade(move |socket| handle(registry, socket, addr))
}

async fn handle(registry: Arc<Registry>, socket: WebSocket, _addr: SocketAddr) {
    let handle = registry.register().await;
    let node_id = handle.id;
    let reply_pool = Arc::new(WSReplyPool::new());
    let (sender, receiver) = socket.split();
    // Task to receive messages from Registry and relay
    // them to the worker.
    let mut worker_relay_task = {
        let reply_pool = reply_pool.clone();
        tokio::spawn(async { worker_relay(sender, handle, reply_pool).await })
    };
    // Task to receive messages from workers and relay
    // them to registry
    let mut registry_relay_task =
        tokio::spawn(async { registry_relay(receiver, reply_pool).await });

    tokio::select! {
        maybe_handle = (&mut worker_relay_task) => {
            match maybe_handle {
                Ok(_) => tracing::info!("Worker relay: Disconnected"),
                _ => {
                }
            }
            registry_relay_task.abort();

        },
        e = (&mut registry_relay_task) => {
            match e {
                Ok(_) => tracing::info!("Registry relay: Disconnected"),
                Err(e) => tracing::warn!("Registry relay: Failed ({e})")
            }
            worker_relay_task.abort();
        },
    }
    registry.deregister(&node_id).await;
}

async fn worker_relay(
    mut socket: SplitSink<WebSocket, Message>,
    mut handle: RegistryHandle,
    reply_pool: Arc<WSReplyPool>,
) -> RegistryHandle {
    let mut cycles = 0;
    loop {
        match handle.receiver.try_recv() {
            Ok(msg) => {
                let ctrl = send_to_worker(msg, &mut socket, &reply_pool).await;
                if ctrl.is_break() {
                    // Worker likely disconnected
                    break;
                }
            }
            Err(_) => {}
        };
        // TODO: make pinging every 100 cycles configurable.
        cycles = (cycles + 1) % 100;
        if cycles == 0 {
            let ping = socket.send(Message::Ping(vec![0x41, 0x42, 0x43])).await;
            if ping.is_err() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    handle
}

async fn send_to_worker(
    msg: NodeMsg,
    socket: &mut SplitSink<WebSocket, Message>,
    reply_pool: &Arc<WSReplyPool>,
) -> ControlFlow<(), ()> {
    tracing::trace!("Relaying message {msg:?} to worker");
    match msg {
        NodeMsg::Invoke { uri, args, sender } => {
            let request_id = Uuid::new_v4().to_string();
            let reply_sender = sender;
            let ws_msg: WSProto = WSProto::invoke_request(request_id.clone(), uri, args);
            // TODO: insert call timestamp to cleanup senders
            reply_pool.register(request_id, reply_sender).await;
            let sent_status = socket.send(Message::Text(ws_msg.to_json())).await;
            if sent_status.is_err() {
                return ControlFlow::Break(());
            }
        }
    }
    ControlFlow::Continue(())
}

async fn registry_relay(mut socket: SplitStream<WebSocket>, reply_pool: Arc<WSReplyPool>) {
    tracing::trace!("Starting registry relay for worker");
    loop {
        tokio::select! {
            maybe_msg = socket.next() => {
                match maybe_msg {
                    // TODO: Ugly
                    Some(Ok(msg)) => {
                        let ctrl = process_worker_msg(msg, &reply_pool).await;
                        if ctrl.is_break() {
                            break;
                        }
                    },
                    Some(Err(msg)) => {
                        tracing::warn!("Error from registry_relay: {:?}", msg);

                    }
                    // TODO: properly handle empty response
                    None => continue,
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                tracing::warn!("Worker hasn't sent a message in 60 seconds, disconnecting..");
                break
            }
        }
    }
}

async fn process_worker_msg(msg: Message, reply_pool: &Arc<WSReplyPool>) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(response_body) => {
            tracing::trace!("Worker responded: {response_body:?}");
            let response = match WSProto::from_json(&response_body) {
                Ok(payload) => payload,
                // WS Client sending malformed payloads
                // Assume malicious and disconnect
                Err(_) => return ControlFlow::Break(()),
            };

            match response {
                WSProto::Result {
                    request_id,
                    content,
                } => {
                    let msg: RegistryMsg = RegistryMsg::InvokeResult(content);
                    let reply_status = reply_pool.reply(&request_id, msg).await;
                    match reply_status {
                        // Likely malicious behavior?
                        // Worker sending return values unprovoked?
                        // disconnect
                        Err(_) => return ControlFlow::Break(()),
                        _ => (),
                    }
                }
                _ => {
                    // WS client sending illegal messages i.e. WSProto::Invoke
                    // disconnect
                    return ControlFlow::Break(());
                }
            }
        }

        Message::Pong(_) => {
            // TODO: Should update worker last_ts
        }

        Message::Close(_) => {
            tracing::debug!("Worker terminated connection");
            return ControlFlow::Break(());
        }

        unhandled => tracing::warn!("Unexpected message from worker: {:?}", unhandled),
    };
    ControlFlow::Continue(())
}
