use super::Storage;
use std::io::Error as IOError;
use std::path::Path;

#[derive(Clone)]
pub struct LocalStorage {
    base_dir: String,
}

impl Storage for LocalStorage {
    fn init() -> Self {
        let dirname = "bin";
        let path = Path::new(dirname);
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        Self {
            base_dir: String::from(dirname),
        }
    }

    fn fetch(&self, name: &str) -> Result<Vec<u8>, IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        std::fs::read(path)
    }

    fn store(&self, name: &str, binary: &[u8]) -> Result<(), IOError> {
        let path = Path::new(&self.base_dir).join(name).with_extension("wasm");
        std::fs::write(path, binary)
    }
}
