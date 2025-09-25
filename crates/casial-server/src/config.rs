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
    pub federation: FederationSettings,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSettings {
    pub enabled: bool,
    pub downstream_servers: Vec<DownstreamMcpServer>,
    pub catalog_refresh_interval: u64,
    pub spec_version_tracking: bool,
    pub connection_timeout_ms: u64,
    pub max_retries: u32,
    #[serde(default = "default_tool_cache_ttl_seconds")]
    pub tool_cache_ttl_seconds: u64,
    #[serde(default = "default_circuit_breaker_threshold")]
    pub circuit_breaker_threshold: u32,
    #[serde(default = "default_circuit_breaker_reset_seconds")]
    pub circuit_breaker_reset_seconds: u64,
    #[serde(default = "default_backoff_initial_ms")]
    pub backoff_initial_ms: u64,
    #[serde(default = "default_backoff_max_ms")]
    pub backoff_max_ms: u64,
}

impl Default for FederationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            downstream_servers: vec![],
            catalog_refresh_interval: 300,
            spec_version_tracking: true,
            connection_timeout_ms: 10_000,
            max_retries: 3,
            tool_cache_ttl_seconds: default_tool_cache_ttl_seconds(),
            circuit_breaker_threshold: default_circuit_breaker_threshold(),
            circuit_breaker_reset_seconds: default_circuit_breaker_reset_seconds(),
            backoff_initial_ms: default_backoff_initial_ms(),
            backoff_max_ms: default_backoff_max_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownstreamMcpServer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub connection_type: String, // "websocket" | "stdio"
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: u8, // For conflict resolution
    pub auth: Option<McpAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuth {
    pub auth_type: String, // "header" | "query" | "websocket-subprotocol"
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
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
            federation: FederationSettings::default(),
        }
    }
}

fn default_tool_cache_ttl_seconds() -> u64 {
    300
}

fn default_circuit_breaker_threshold() -> u32 {
    3
}

fn default_circuit_breaker_reset_seconds() -> u64 {
    180
}

fn default_backoff_initial_ms() -> u64 {
    250
}

fn default_backoff_max_ms() -> u64 {
    5_000
}

impl ServerConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let config: ServerConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
