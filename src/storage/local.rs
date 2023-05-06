use super::Storage;
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

impl Storage for LocalStorage {
    fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        std::fs::read(path)
    }

    fn store(&self, name: &str, binary: &[u8], meta: String) -> Result<(), IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        let meta_path = Path::new(&self.base_dir).join(name).with_extension("json");
        std::fs::write(path, binary)?;
        std::fs::write(meta_path, meta.as_bytes())
    }
}
