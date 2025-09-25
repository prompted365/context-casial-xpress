//! Global Pitfall Avoidance Shim
//!
//! Provides quality-of-life defaults and context injection for all MCP tool calls.
//! This shim automatically injects helpful context like current dates, timestamps,
//! and other QoL enhancements to prevent common AI pitfalls.

use anyhow::Result;
use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Configuration for the pitfall avoidance shim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimConfig {
    /// Enable the shim (can be toggled via --shim flag)
    pub enabled: bool,

    /// Inject current date/time by default
    pub inject_datetime: bool,

    /// Add timestamps to all returns
    pub timestamp_returns: bool,

    /// Custom extension string (via --shim-extend "...")
    pub custom_extension: Option<String>,

    /// Additional QoL features
    pub features: ShimFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimFeatures {
    /// Inject timezone information
    pub inject_timezone: bool,

    /// Add execution metadata
    pub add_execution_metadata: bool,

    /// Include system information
    pub include_system_info: bool,

    /// Provide format hints for dates
    pub date_format_hints: bool,

    /// Add helpful context about common pitfalls
    pub pitfall_warnings: bool,
}

impl Default for ShimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inject_datetime: true,
            timestamp_returns: true,
            custom_extension: None,
            features: ShimFeatures::default(),
        }
    }
}

impl Default for ShimFeatures {
    fn default() -> Self {
        Self {
            inject_timezone: true,
            add_execution_metadata: true,
            include_system_info: false,
            date_format_hints: true,
            pitfall_warnings: true,
        }
    }
}

/// Global pitfall avoidance shim that processes tool calls
pub struct PitfallAvoidanceShim {
    config: ShimConfig,
}

impl PitfallAvoidanceShim {
    /// Create a new shim with the given configuration
    pub fn new(config: ShimConfig) -> Self {
        Self { config }
    }

    /// Create from command-line arguments
    pub fn from_args(enabled: bool, extension: Option<String>) -> Self {
        let mut config = ShimConfig::default();
        config.enabled = enabled;
        config.custom_extension = extension;

        Self { config }
    }

    /// Check if the shim is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Augment tool request with pitfall avoidance context
    pub fn augment_request(
        &self,
        tool_name: &str,
        original_params: &Value,
        agent_role: Option<&str>,
    ) -> Result<Value> {
        if !self.config.enabled {
            return Ok(original_params.clone());
        }

        debug!("Augmenting request for tool: {}", tool_name);

        let mut augmented = if original_params.is_object() {
            original_params.as_object().unwrap().clone()
        } else {
            serde_json::Map::new()
        };

        // Add shim context
        let mut shim_context = serde_json::Map::new();

        // Inject current date/time
        if self.config.inject_datetime {
            let now_utc = Utc::now();
            let now_local = Local::now();

            shim_context.insert(
                "current_datetime_utc".to_string(),
                json!(now_utc.to_rfc3339()),
            );
            shim_context.insert(
                "current_datetime_local".to_string(),
                json!(now_local.to_rfc3339()),
            );
            shim_context.insert(
                "current_date".to_string(),
                json!(now_local.format("%Y-%m-%d").to_string()),
            );
            shim_context.insert(
                "current_time".to_string(),
                json!(now_local.format("%H:%M:%S").to_string()),
            );

            if self.config.features.inject_timezone {
                shim_context.insert(
                    "timezone".to_string(),
                    json!(now_local.format("%Z").to_string()),
                );
                shim_context.insert(
                    "timezone_offset".to_string(),
                    json!(now_local.format("%z").to_string()),
                );
            }

            if self.config.features.date_format_hints {
                shim_context.insert(
                    "date_format_hints".to_string(),
                    json!({
                        "iso8601": now_utc.to_rfc3339(),
                        "unix_timestamp": now_utc.timestamp(),
                        "human_readable": now_local.format("%B %d, %Y at %I:%M %p %Z").to_string(),
                        "sortable": now_local.format("%Y%m%d_%H%M%S").to_string()
                    }),
                );
            }
        }

        // Add execution metadata
        if self.config.features.add_execution_metadata {
            let mut metadata = serde_json::Map::new();
            metadata.insert("tool_name".to_string(), json!(tool_name));
            metadata.insert("shim_version".to_string(), json!("1.0.0"));
            metadata.insert(
                "request_id".to_string(),
                json!(uuid::Uuid::new_v4().to_string()),
            );
            metadata.insert(
                "timestamp".to_string(),
                json!(Utc::now().timestamp_millis()),
            );

            if let Some(role) = agent_role {
                metadata.insert("agent_role".to_string(), json!(role));
            }

            shim_context.insert("execution_metadata".to_string(), json!(metadata));
        }

        // Include system information if enabled
        if self.config.features.include_system_info {
            shim_context.insert(
                "system_info".to_string(),
                json!({
                    "platform": std::env::consts::OS,
                    "arch": std::env::consts::ARCH,
                    "hostname": hostname::get().unwrap_or_default().to_string_lossy()
                }),
            );
        }

        // Add pitfall warnings
        if self.config.features.pitfall_warnings {
            let warnings = self.get_contextual_warnings(tool_name, agent_role);
            if !warnings.is_empty() {
                shim_context.insert("pitfall_warnings".to_string(), json!(warnings));
            }
        }

        // Add custom extension if provided
        if let Some(ref extension) = self.config.custom_extension {
            shim_context.insert("custom_extension".to_string(), json!(extension));
        }

        // Inject shim context
        augmented.insert("_shim_context".to_string(), json!(shim_context));

        info!("Request augmented with {} shim fields", shim_context.len());

        Ok(json!(augmented))
    }

    /// Process tool response with timestamp and metadata
    pub fn process_response(&self, tool_name: &str, original_response: &Value) -> Result<Value> {
        if !self.config.enabled || !self.config.timestamp_returns {
            return Ok(original_response.clone());
        }

        debug!("Processing response for tool: {}", tool_name);

        let mut processed = if original_response.is_object() {
            original_response.as_object().unwrap().clone()
        } else {
            let mut map = serde_json::Map::new();
            map.insert("result".to_string(), original_response.clone());
            map
        };

        // Add response metadata
        let response_metadata = json!({
            "processed_at": Utc::now().to_rfc3339(),
            "processing_time_ms": 0, // Would be calculated from actual timing
            "tool_name": tool_name,
            "shim_applied": true
        });

        processed.insert("_response_metadata".to_string(), response_metadata);

        Ok(json!(processed))
    }

    /// Get contextual warnings for specific tools
    fn get_contextual_warnings(&self, tool_name: &str, agent_role: Option<&str>) -> Vec<String> {
        let mut warnings = Vec::new();

        // Add general warnings
        warnings.push(format!(
            "Current date is {} - ensure any date-based queries use this as reference",
            Local::now().format("%Y-%m-%d")
        ));

        // Tool-specific warnings
        match tool_name {
            "web_search_exa" | "deep_researcher_start" => {
                warnings.push("When searching for recent events, remember to include the current year in queries".to_string());
                warnings.push("For documentation searches, prefer recent versions by including year constraints".to_string());
            }
            tool if tool.starts_with("exa.") => {
                warnings.push("When searching for recent events, remember to include the current year in queries".to_string());
                warnings.push("For documentation searches, prefer recent versions by including year constraints".to_string());

                // Exa-specific warnings based on tool
                if tool_name.starts_with("exa.research") {
                    warnings.push(
                        "Research tasks should define clear output schemas for structured results"
                            .to_string(),
                    );
                    warnings.push("Use multiple query variations to maximize coverage".to_string());
                }

                if tool_name == "exa.websets.create" {
                    warnings.push(
                        "Configure webhooks for real-time notifications on standing queries"
                            .to_string(),
                    );
                    warnings
                        .push("Use semantic deduplication to prevent alert fatigue".to_string());
                }

                if tool_name == "exa.get_contents" {
                    warnings.push(
                        "Consider using livecrawl='preferred' for time-sensitive content"
                            .to_string(),
                    );
                    warnings.push(
                        "Enable subpages exploration for comprehensive site coverage".to_string(),
                    );
                }
            }
            "orchestrate_mcp_proxy" => {
                warnings.push(
                    "Ensure target server URLs are properly validated before proxying".to_string(),
                );
                warnings.push(
                    "Consider consciousness coordination impacts on downstream servers".to_string(),
                );
            }
            _ => {}
        }

        // Agent role-specific warnings
        if let Some(role) = agent_role {
            match role {
                "researcher" | "analyst" => {
                    warnings.push("Maintain evidence chains and cite all sources".to_string());
                    warnings
                        .push("Surface contradictions and uncertainties explicitly".to_string());
                }
                "monitor" | "watcher" => {
                    warnings.push(
                        "Set up appropriate update frequencies for monitoring tasks".to_string(),
                    );
                    warnings
                        .push("Use domain filters to reduce noise in standing queries".to_string());
                }
                "orchestrator" => {
                    warnings.push(
                        "Coordinate multi-agent patterns for complex research tasks".to_string(),
                    );
                    warnings.push("Ensure proper task decomposition before execution".to_string());
                }
                _ => {}
            }
        }

        warnings
    }

    /// Get current configuration (for display/editing)
    pub fn get_config(&self) -> &ShimConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ShimConfig) {
        self.config = config;
    }

    /// Export configuration as JSON
    pub fn export_config(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.config)?)
    }

    /// Import configuration from JSON
    pub fn import_config(&mut self, json: &str) -> Result<()> {
        self.config = serde_json::from_str(json)?;
        Ok(())
    }
}

impl Default for PitfallAvoidanceShim {
    fn default() -> Self {
        Self::new(ShimConfig::default())
    }
}

/// Command-line arguments for shim configuration
#[derive(Debug, Clone)]
pub struct ShimArgs {
    /// Enable/disable the shim (--shim or --no-shim)
    pub enabled: bool,

    /// Custom extension string (--shim-extend "...")
    pub extension: Option<String>,

    /// Path to custom shim config (--shim-config path/to/config.json)
    pub config_file: Option<String>,
}

impl Default for ShimArgs {
    fn default() -> Self {
        Self {
            enabled: true,
            extension: None,
            config_file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shim_config() {
        let config = ShimConfig::default();
        assert!(config.enabled);
        assert!(config.inject_datetime);
        assert!(config.timestamp_returns);
    }

    #[test]
    fn test_augment_request() {
        let shim = PitfallAvoidanceShim::new(ShimConfig::default());
        let original = json!({"query": "test"});
        let augmented = shim.augment_request("test_tool", &original, None).unwrap();

        assert!(augmented["_shim_context"].is_object());
        assert!(augmented["_shim_context"]["current_date"].is_string());
        assert!(augmented["query"].is_string());
    }

    #[test]
    fn test_process_response() {
        let shim = PitfallAvoidanceShim::new(ShimConfig::default());
        let original = json!({"status": "success"});
        let processed = shim.process_response("test_tool", &original).unwrap();

        assert!(processed["_response_metadata"].is_object());
        assert!(processed["_response_metadata"]["processed_at"].is_string());
        assert_eq!(processed["status"], "success");
    }

    #[test]
    fn test_disabled_shim() {
        let mut config = ShimConfig::default();
        config.enabled = false;
        let shim = PitfallAvoidanceShim::new(config);

        let original = json!({"query": "test"});
        let augmented = shim.augment_request("test_tool", &original, None).unwrap();
        assert_eq!(augmented, original);
    }
}
