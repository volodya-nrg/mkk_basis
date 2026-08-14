use config::{Config as ConfigExternal, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub service_name: String,
    pub version: String,
    pub private_key_path: String,
    pub log: Log,
    pub postgres: Postgres,
    pub http_server: HTTPServer,
}
#[derive(Deserialize)]
pub struct Log {
    pub level: String,
    pub filepath: Option<String>,
}
#[derive(Deserialize)]
pub struct Postgres {
    pub dsn: String,
}
#[derive(Deserialize)]
pub struct HTTPServer {
    pub address: String,
    pub tls: Tls,
}
#[derive(Deserialize)]
pub struct Tls {
    pub is_use: bool,
    pub ca_filepath: String,
    pub crt_filepath: String,
    pub key_filepath: String,
}

impl Config {
    pub fn new(filepath: &str) -> Result<Self, String> {
        let config_builder =
            ConfigExternal::builder().add_source(File::new(filepath, FileFormat::Yaml));
        let result = config_builder
            .build()
            .map_err(|e| format!("failed to build: {e}"))?;
        let config: Config = result
            .try_deserialize()
            .map_err(|e| format!("failed to deserialize: {e}"))?;

        Ok(config)
    }
}
