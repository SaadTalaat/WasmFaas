use crate::storage::StorageInterface;
use crate::Registry;
use std::sync::Arc;

#[derive(Clone)]
pub struct Handles {
    pub storage: Arc<StorageInterface>,
    pub registry: Arc<Registry>,
}

impl Handles {
    pub fn new(registry: Registry) -> Self {
        Self {
            storage: Arc::new(StorageInterface::new()),
            registry: Arc::new(registry),
        }
    }
}
