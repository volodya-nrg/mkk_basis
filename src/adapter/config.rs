use config::{Config as ConfigExternal, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config error: {0}")]
    FailedConfig(#[from] config::ConfigError),
}

#[derive(Deserialize)]
pub struct Config {
    pub service_name: String,
    pub version: String,
    pub private_key_path: String,
    pub addr: String, // example: http://localhost.loc
    pub log: Log,
    pub postgres: Postgres,
    pub http_server: HTTPServer,
    pub email: Email,
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
#[derive(Deserialize)]
pub struct Email {
    pub host: String,
    pub login: String,
    pub pass: String,
    pub from_email: String,
    pub from_name: String,
}

impl Config {
    pub fn new(filepath: &str) -> Result<Self, ConfigError> {
        Ok(ConfigExternal::builder()
            .add_source(File::new(filepath, FileFormat::Yaml))
            .build()?
            .try_deserialize::<Self>()?)
    }
}
