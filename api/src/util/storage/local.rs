use super::Storage;
use axum::async_trait;
use std::io::Error as IOError;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LocalStorage {
    base_dir: String,
}

impl LocalStorage {
    pub fn new(directory: String) -> Self {
        let path = Path::new(&directory);
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        Self {
            base_dir: directory,
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        tokio::fs::read(path).await
    }

    async fn store(&self, name: &str, binary: &[u8]) -> Result<String, IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        let path_str = format!("{}", path.display());
        tokio::fs::write(path, binary).await?;
        Ok(path_str)
    }
}
