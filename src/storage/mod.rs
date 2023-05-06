mod local;
use crate::settings::{StorageKind, StorageSettings};
pub use local::LocalStorage;
use std::io::Error as IOError;

pub trait Storage {
    fn store(&self, name: &str, binary: &[u8], metadata: String) -> Result<(), IOError>;
    fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError>;
}

pub fn init(settings: StorageSettings) -> impl Storage {
    match settings.medium {
        StorageKind::Local { directory } => LocalStorage::new(directory),
    }
}
