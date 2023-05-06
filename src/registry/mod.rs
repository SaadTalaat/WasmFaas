use crate::proto::{NodeMsg, RegistryMsg};
use crate::settings::RegistrySettings;
use crate::status::{Status, StatusKind};
use axum::response::{IntoResponse, Response};
use serde_json::Value as JsValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc::{channel, error::SendTimeoutError, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub struct Registry {
    nodes: NodeBucket,
    channel_size: usize,
    timeout: Duration,
}

impl Registry {
    pub fn start(settings: RegistrySettings) -> Self {
        Self {
            nodes: NodeBucket::new(),
            channel_size: settings.channel_size,
            timeout: Duration::from_secs(settings.timeout_secs as u64),
        }
    }

    pub async fn register(&self) -> RegistryHandle {
        let node_id = Uuid::new_v4();
        tracing::trace!("Registering new worker: {node_id}");
        let (sender, receiver) = channel(self.channel_size);
        let node = NodeHandle::new(node_id, sender);
        self.nodes.add(node).await;
        RegistryHandle::new(node_id, receiver)
    }

    pub async fn deregister(&self, node_id: &Uuid) {
        tracing::trace!("Deregistering worker: {node_id}");
        let maybe_handle = self.nodes.remove(node_id).await;
        match maybe_handle {
            Ok(_) => {}
            Err(e) => tracing::warn!("Error while deregistering node: {node_id}, Error: {e}"),
        }
    }

    pub async fn invoke(
        &self,
        fn_name: String,
        args: Vec<JsValue>,
    ) -> Result<Status, BackendError> {
        let node = self.nodes.get_node().await?;
        let node_id = node.lock().await.id;
        tracing::trace!("Invoking function ({fn_name}) on worker ({node_id})");

        let (sender, mut receiver) = oneshot::channel();
        let msg = NodeMsg::Invoke {
            name: fn_name,
            args,
            sender,
        };

        node.lock()
            .await
            .sender
            .send_timeout(msg, self.timeout)
            .await?;
        let mut rcv_return_task = tokio::spawn(async move {
            loop {
                let msg = receiver.try_recv();
                match msg {
                    Ok(m) => return Ok(m),
                    _ => tokio::time::sleep(Duration::from_millis(5)).await,
                }
            }
            // Unreachable
            Err(BackendError::Timeout)
        });

        let result_v = tokio::select! {
            // XXX REPLACE ME
            ret_val = (&mut rcv_return_task) => {
                match ret_val {
                    Ok(r) => r,
                    Err(_) => {
                        tracing::warn!("Worker: {:?} malformed reply", node_id);
                        return Err(BackendError::NoReply)
                    }
                }
            },
            _ = tokio::time::sleep(self.timeout) => {
                rcv_return_task.abort();
                return Err(BackendError::Timeout)
            }
        }?;

        match result_v {
            RegistryMsg::InvokeResult(r) => Ok(Status::ok_payload(r)),
        }
    }
}

pub struct RegistryHandle {
    pub id: Uuid,
    pub receiver: Receiver<NodeMsg>,
}

impl RegistryHandle {
    pub fn new(id: Uuid, receiver: Receiver<NodeMsg>) -> Self {
        Self { id, receiver }
    }
}

pub struct NodeHandle {
    pub id: Uuid,
    pub sender: Arc<Sender<NodeMsg>>,
}

impl NodeHandle {
    pub fn new(id: Uuid, sender: Sender<NodeMsg>) -> Self {
        Self {
            id,
            sender: Arc::new(sender),
        }
    }
}

impl NodeBucket {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(vec![]),
            key_to_idx: RwLock::new(HashMap::new()),
            counter: RwLock::new(0),
        }
    }

    pub async fn add(&self, node: NodeHandle) {
        let mut nodes = self.nodes.write().await;
        self.key_to_idx
            .write()
            .await
            .insert(node.id.clone(), nodes.len());
        nodes.push(Arc::new(Mutex::new(node)))
    }

    pub async fn remove(&self, node_id: &Uuid) -> Result<Arc<Mutex<NodeHandle>>, BackendError> {
        let mut nodes = self.nodes.write().await;
        let idx = self
            .key_to_idx
            .write()
            .await
            .remove(node_id)
            .ok_or(BackendError::InternalNodeHandling)?;
        let node = nodes.remove(idx);
        Ok(node)
    }

    pub async fn get_node(&self) -> Result<Arc<Mutex<NodeHandle>>, BackendError> {
        let nodes = self.nodes.read().await;
        if nodes.len() == 0 {
            Err(BackendError::NoWorkersAvailable)
        } else {
            let mut counter = self.counter.write().await;
            let node = nodes
                .get(*counter)
                .ok_or(BackendError::InternalNodeHandling)?;
            // Wrap around
            *counter = (*counter + 1) % nodes.len();
            Ok(node.clone())
        }
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Out of resources")]
    NoWorkersAvailable,
    #[error("Timed out while invoking function")]
    Timeout,
    #[error("encountered error while invoking function")]
    NoReply,
    #[error("encountered internal error while invoking function")]
    InternalNodeHandling,
}

impl Into<Status> for BackendError {
    fn into(self) -> Status {
        let msg = format!("{}", self);
        let kind = match self {
            _ => StatusKind::InternalError,
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

struct NodeBucket {
    nodes: RwLock<Vec<Arc<Mutex<NodeHandle>>>>,
    key_to_idx: RwLock<HashMap<Uuid, usize>>,
    counter: RwLock<usize>,
}
