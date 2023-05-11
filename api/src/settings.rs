use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub http: HttpSettings,
    pub storage: StorageSettings,
    pub compiler: CompilerSettings,
    pub registry: RegistrySettings,
    pub db_url: String,
}

#[derive(Debug, Deserialize)]
pub struct HttpSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StorageKind {
    Local { directory: String },
}

#[derive(Debug, Deserialize)]
pub struct StorageSettings {
    pub medium: StorageKind,
}

#[derive(Debug, Deserialize)]
pub struct CompilerSettings {
    pub source_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct RegistrySettings {
    pub channel_size: usize,
    pub timeout_secs: usize,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("./config/default.toml"))
            .add_source(config::Environment::with_prefix("FAAS"))
            .build()?;
        return config.try_deserialize();
    }
}
