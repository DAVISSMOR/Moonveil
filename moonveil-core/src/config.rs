use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_transport")]
    pub transport: String,
}

fn default_transport() -> String {
    "tcp".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7878,
            transport: default_transport(),
        }
    }
}

impl Config {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn from_toml_str(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| Error::Config(e.to_string()))
    }

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::Config(format!("failed to read {}: {e}", path.as_ref().display())))?;
        Self::from_toml_str(&content)
    }

    /// Load from `MOONVEIL_CONFIG` env var, or fall back to defaults.
    pub fn load() -> Self {
        std::env::var("MOONVEIL_CONFIG")
            .ok()
            .and_then(|path| Self::from_toml_file(path).ok())
            .unwrap_or_default()
    }
}
