use super::error::BackendError;
use crate::proto::WorkerMsg;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex, RwLock};
use uuid::Uuid;

pub struct WorkerHandle {
    pub id: Uuid,
    pub sender: Arc<Sender<WorkerMsg>>,
}

impl WorkerHandle {
    pub fn new(id: Uuid, sender: Sender<WorkerMsg>) -> Self {
        Self {
            id,
            sender: Arc::new(sender),
        }
    }
}

pub struct WorkersBucket {
    workers: RwLock<Vec<Arc<Mutex<WorkerHandle>>>>,
    key_to_idx: RwLock<HashMap<Uuid, usize>>,
    counter: RwLock<usize>,
}

impl WorkersBucket {
    pub fn new() -> Self {
        Self {
            workers: RwLock::new(vec![]),
            key_to_idx: RwLock::new(HashMap::new()),
            counter: RwLock::new(0),
        }
    }

    pub async fn add(&self, worker: WorkerHandle) {
        let mut workers = self.workers.write().await;
        self.key_to_idx
            .write()
            .await
            .insert(worker.id.clone(), workers.len());
        workers.push(Arc::new(Mutex::new(worker)))
    }

    pub async fn remove(&self, worker_id: &Uuid) -> Result<Arc<Mutex<WorkerHandle>>, BackendError> {
        let mut workers = self.workers.write().await;
        let idx = self
            .key_to_idx
            .write()
            .await
            .remove(worker_id)
            .ok_or(BackendError::InternalNodeHandling)?;
        let worker = workers.remove(idx);
        Ok(worker)
    }

    pub async fn get_worker(&self) -> Result<Arc<Mutex<WorkerHandle>>, BackendError> {
        let workers = self.workers.read().await;
        if workers.len() == 0 {
            Err(BackendError::NoWorkersAvailable)
        } else {
            let mut counter = self.counter.write().await;
            let worker = workers
                .get(*counter)
                .ok_or(BackendError::InternalNodeHandling)?;
            // Wrap around
            *counter = (*counter + 1) % workers.len();
            Ok(worker.clone())
        }
    }
}
