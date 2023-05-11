use crate::util::{
    compiler::Compiler,
    storage::{self, Storage},
};
use crate::{registry::Registry, Settings};
use std::error::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct Handles {
    pub storage: Arc<dyn Storage + Sync + Send>,
    pub registry: Arc<Registry>,
    pub compiler: Arc<Compiler>,
}

impl Handles {
    pub fn new(settings: &Settings) -> Result<Self, Box<dyn Error>> {
        let registry = Registry::start(&settings.registry);
        let compiler = Compiler::new(&settings.compiler.source_dir);
        let storage = storage::init(&settings.storage);

        let instance = Self {
            storage: Arc::new(storage),
            registry: Arc::new(registry),
            compiler: Arc::new(compiler),
        };
        Ok(instance)
    }
}
