mod local;
pub use local::LocalStorage;

use std::io::Error as IOError;

pub trait Storage {
    fn init() -> Self;
    fn store(&self, name: &str, binary: &[u8]) -> Result<(), IOError>;
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

    pub fn store(&self, name: &str, binary: &[u8]) -> Result<(), IOError> {
        self.medium.store(name, binary)
    }

    pub fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError> {
        self.medium.fetch(name)
    }
}
