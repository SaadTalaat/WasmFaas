use crate::compiler::Compiler;
use crate::storage;
use crate::{Registry, Settings};
use std::error::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct Handles {
    pub storage: Arc<dyn storage::Storage + Sync + Send>,
    pub registry: Arc<Registry>,
    pub compiler: Arc<Compiler>,
}

impl Handles {
    pub fn new(registry: Registry, compiler: Compiler) -> Result<Self, Box<dyn Error>> {
        let settings = Settings::new()?;
        let instance = Self {
            storage: Arc::new(storage::init(settings.storage)),
            registry: Arc::new(registry),
            compiler: Arc::new(compiler),
        };
        Ok(instance)
    }
}
