use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { host: default_server_host(), port: default_server_port() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    #[serde(default = "default_preshared_key")]
    pub preshared_key: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self { preshared_key: default_preshared_key() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default = "default_transport_mode")]
    pub mode: String,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self { mode: default_transport_mode() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { level: default_log_level() }
    }
}

fn default_server_host() -> String { "127.0.0.1".to_string() }
fn default_server_port() -> u16 { 7878 }
fn default_preshared_key() -> String {
    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
}
fn default_transport_mode() -> String { "tcp".to_string() }
fn default_log_level() -> String { "info".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonveilConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub crypto: CryptoConfig,
    #[serde(default)]
    pub transport: TransportConfig,
    #[serde(default)]
    pub log: LogConfig,
}

impl Default for MoonveilConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            crypto: CryptoConfig::default(),
            transport: TransportConfig::default(),
            log: LogConfig::default(),
        }
    }
}

impl MoonveilConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(e.to_string()))?;
        toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::Config(format!("failed to read {}: {e}", path.as_ref().display())))?;
        toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn load() -> Self {
        std::env::var("MOONVEIL_CONFIG")
            .ok()
            .and_then(|path| Self::from_toml_file(path).ok())
            .unwrap_or_default()
    }
}

pub type Config = MoonveilConfig;