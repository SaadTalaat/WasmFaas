use super::worker::{WorkerHandle, WorkersBucket};
use super::error::{BackendError};

use crate::{
    db::models::FunctionType,
    proto::{WorkerMsg, RegistryMsg},
    settings::RegistrySettings,
    status::Status,
};
use serde_json::Value as JsValue;
use tokio::{
    sync::{
        mpsc::{channel, Receiver},
        oneshot,
    },
    time::Duration,
};
use uuid::Uuid;

pub struct Registry {
    workers: WorkersBucket,
    channel_size: usize,
    timeout: Duration,
}

impl Registry {
    pub fn start(settings: &RegistrySettings) -> Self {
        Self {
            workers: WorkersBucket::new(),
            channel_size: settings.channel_size,
            timeout: Duration::from_secs(settings.timeout_secs as u64),
        }
    }

    pub async fn register(&self) -> RegistryHandle {
        let worker_id = Uuid::new_v4();
        tracing::trace!("Registering new worker: {worker_id}");
        let (sender, receiver) = channel(self.channel_size);
        let worker = WorkerHandle::new(worker_id, sender);
        self.workers.add(worker).await;
        RegistryHandle::new(worker_id, receiver)
    }

    pub async fn deregister(&self, worker_id: &Uuid) {
        tracing::trace!("Deregistering worker: {worker_id}");
        let maybe_handle = self.workers.remove(worker_id).await;
        match maybe_handle {
            Ok(_) => {}
            Err(e) => tracing::warn!("Error while deregistering worker: {worker_id}, Error: {e}"),
        }
    }

    pub async fn invoke(
        &self,
        name: String,
        uri: String,
        signature: FunctionType,
        args: Vec<JsValue>,
    ) -> Result<Status, BackendError> {
        let worker = self.workers.get_worker().await?;
        let worker_id = worker.lock().await.id;
        tracing::trace!("Invoking function ({uri}) on worker ({worker_id})");

        let (sender, mut receiver) = oneshot::channel();
        let msg = WorkerMsg::Invoke {
            name,
            uri,
            signature,
            args,
            sender,
        };

        worker.lock()
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

            // NOTE: Unreachable, but needed for type inference.
            Err(BackendError::Timeout)
        });

        let result_v = tokio::select! {
            // XXX REPLACE ME
            ret_val = (&mut rcv_return_task) => {
                match ret_val {
                    Ok(r) => r,
                    Err(_) => {
                        tracing::warn!("Worker: {:?} malformed reply", worker_id);
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
            _ => panic!("Unexpected RegistryMsg received {:?}", result_v)
        }
    }
}

pub struct RegistryHandle {
    pub id: Uuid,
    pub receiver: Receiver<WorkerMsg>,
}

impl RegistryHandle {
    pub fn new(id: Uuid, receiver: Receiver<WorkerMsg>) -> Self {
        Self { id, receiver }
    }
}
