use crate::storage::StorageInterface;
use std::sync::Arc;

#[derive(Clone)]
pub struct Handles {
    pub storage: Arc<StorageInterface>,
}

impl Handles {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(StorageInterface::new()),
        }
    }
}
