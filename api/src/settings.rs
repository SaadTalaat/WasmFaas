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
    pub assets_directory: String,
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
        let config_base = std::env::var("FAAS_CONFIG_DIR").unwrap_or_else(|_| "./config".into());
        let mode = std::env::var("FAAS_ENV").unwrap_or_else(|_| "dev".into());
        let config = Config::builder()
            .add_source(File::with_name(&format!("{}/{}.toml", config_base, mode)))
            .add_source(config::Environment::with_prefix("FAAS"))
            .build()?;
        return config.try_deserialize();
    }
}
