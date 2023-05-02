mod local;
pub use local::LocalStorage;

use std::io::Error as IOError;

pub trait Storage {
    fn init() -> Self;
    fn store(&self, name: &str, binary: &[u8], metadata: String) -> Result<(), IOError>;
    fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError>;
}

#[derive(Clone)]
pub struct StorageInterface {
    medium: LocalStorage,
}

impl StorageInterface {
    pub fn new() -> Self {
        Self {
            medium: LocalStorage::init(),
        }
    }

    pub fn store(&self, name: &str, binary: &[u8], meta: String) -> Result<(), IOError> {
        self.medium.store(name, binary, meta)
    }

    pub fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError> {
        self.medium.fetch(name)
    }
}
