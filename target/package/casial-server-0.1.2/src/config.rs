//! # Server Configuration
//!
//! Configuration management for the Casial server.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub consciousness: ConsciousnessSettings,
    pub metrics: MetricsSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub port: u16,
    pub max_connections: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessSettings {
    pub enabled: bool,
    pub perception_lock_timeout: u64,
    pub paradox_resolution_timeout: u64,
    pub substrate_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSettings {
    pub enabled: bool,
    pub collection_interval: u64,
    pub retention_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
    pub json_format: bool,
    pub file_output: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                port: 8000,
                max_connections: 1000,
                timeout_seconds: 300,
            },
            consciousness: ConsciousnessSettings {
                enabled: true,
                perception_lock_timeout: 30,
                paradox_resolution_timeout: 60,
                substrate_integration: true,
            },
            metrics: MetricsSettings {
                enabled: true,
                collection_interval: 30,
                retention_hours: 24,
            },
            logging: LoggingSettings {
                level: "info".to_string(),
                json_format: false,
                file_output: None,
            },
        }
    }
}

impl ServerConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let config: ServerConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
