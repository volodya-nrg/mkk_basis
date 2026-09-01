use config::{Config as ConfigExternal, File, FileFormat};
use serde::Deserialize;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    Common(String),
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Common(s) => write!(f, "{s}"),
        }
    }
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
        ConfigExternal::builder()
            .add_source(File::new(filepath, FileFormat::Yaml))
            .build()
            .map_err(|e| ConfigError::Common(format!("failed to build: {e}")))?
            .try_deserialize()
            .map_err(|e| ConfigError::Common(format!("failed to deserialize: {e}")))
    }
}
