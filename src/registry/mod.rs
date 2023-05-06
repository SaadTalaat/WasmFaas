mod proto;
pub use proto::{RegistryMsg, NodeMsg};
use crate::status::{Status, StatusKind};
use thiserror::Error;
use std::sync::{Arc};
use tokio::sync::{RwLock, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender, error::{SendTimeoutError}};
use crate::settings::RegistrySettings;
use serde_json::Value as JsValue;
use axum::response::{IntoResponse, Response};
use std::time::Duration;

pub struct Registry {
    nodes: RwLock<Vec<Mutex<NodeHandle>>>,
    counter: RwLock<usize>,
    channel_size :usize,
    timeout: Duration
}

impl Registry {
    pub fn start(settings: RegistrySettings) -> Self {
        Self {
            counter: RwLock::new(0),
            nodes: RwLock::new(vec![]),
            channel_size: settings.channel_size,
            timeout: Duration::from_secs(settings.timeout_secs as u64)
        }
    }

    pub async fn register(&self) -> RegistryHandle {
        let mut nodes = self.nodes.write().await;
        let id = nodes.len();
        let (sender_1, receiver_1) = channel(self.channel_size);
        let (sender_2, receiver_2) = channel(self.channel_size);
        nodes.push(Mutex::new(NodeHandle::new(id, sender_1, receiver_2)));
        RegistryHandle::new(id, sender_2, receiver_1)
    }

    pub async fn disconnect(&self, handle: RegistryHandle) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(handle.id);
    }

    pub async fn invoke(&self, fn_name: String, args: Vec<JsValue>) -> Result<Status, BackendError> {
        let nodes = self.nodes.read().await;
        if nodes.len() == 0 {
            return Err(BackendError::NoWorkersAvailable);
        }
        let mut counter = self.counter.write().await;
        let idx = counter.clone();
        *counter = (idx + 1) % nodes.len();
        let node = nodes.get(idx).unwrap().lock().await;
        let msg = NodeMsg::Invoke(fn_name, args);
        node.sender.send_timeout(msg, self.timeout).await?;
        let mut receiver = node.receiver.lock().await;
        let result = tokio::select! {
            result = receiver.recv() => {
                match result {
                    Some(r) => r,
                    None => {
                        tracing::warn!("Worker: {:?} malformed reply", node.id);
                        return Err(BackendError::NoReply)
                    }
                }
            },
            _ = tokio::time::sleep(self.timeout) => {
                return Err(BackendError::Timeout)
            }
        };

        match result {
            RegistryMsg::InvokeResult(r) => Ok(Status::ok_payload(r)),
        }
    }
}

pub struct RegistryHandle {
    id: usize,
    pub sender: Sender<RegistryMsg>,
    pub receiver: Receiver<NodeMsg>,
}

impl RegistryHandle {
    pub fn new(id: usize, sender: Sender<RegistryMsg>, receiver: Receiver<NodeMsg>) -> Self {
        Self { id, sender, receiver }
    }
}

pub struct NodeHandle {
    pub id: usize,
    pub sender: Arc<Sender<NodeMsg>>,
    pub receiver: Mutex<Receiver<RegistryMsg>>
}

impl NodeHandle {
    pub fn new(id: usize, sender: Sender<NodeMsg>, receiver: Receiver<RegistryMsg>) -> Self {
        Self { id, sender:Arc::new(sender), receiver: Mutex::new(receiver)}
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Out of resources")]
    NoWorkersAvailable,
    #[error("Timed out while invoking function")]
    Timeout,
    #[error("encountered error while invoking function")]
    NoReply
}

impl Into<Status> for BackendError {
    fn into(self) -> Status {
        let msg = format!("{}", self);
        let kind = match self {
            _ => StatusKind::InternalError
        };
        Status::new(kind, msg)
    }
}

impl<T> From<SendTimeoutError<T>> for BackendError {
    fn from(_: SendTimeoutError<T>) -> BackendError {
        BackendError::Timeout
    }
}

impl IntoResponse for BackendError {
    fn into_response(self) -> Response {
        let status: Status = self.into();
        status.into_response()
    }
}
