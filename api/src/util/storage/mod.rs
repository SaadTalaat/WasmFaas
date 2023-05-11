mod local;
use crate::settings::{StorageKind, StorageSettings};
use axum::async_trait;
pub use local::LocalStorage;
use std::io::Error as IOError;

#[async_trait]
pub trait Storage {
    async fn store(&self, name: &str, binary: &[u8]) -> Result<String, IOError>;
    async fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError>;
}

pub fn init(settings: &StorageSettings) -> impl Storage {
    match &settings.medium {
        StorageKind::Local { directory } => LocalStorage::new(directory.clone()),
    }
}
