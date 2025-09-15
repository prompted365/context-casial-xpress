//! # Tool Registry
//!
//! Centralized tool specification registry supporting both local and federated tools.

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Tool specification with federation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub source: ToolSource,
    pub spec_version: String,
    pub spec_hash: String,
    pub last_updated: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Source of tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolSource {
    Local,
    Federated { server_id: String, server_url: String },
}

/// Tool registry for managing local and federated tools
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<DashMap<String, Arc<ToolSpec>>>,
    change_listeners: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<RegistryChangeEvent>>>>,
    metrics: Arc<RwLock<RegistryMetrics>>,
}

/// Registry change events for notifications
#[derive(Debug, Clone)]
pub enum RegistryChangeEvent {
    ToolAdded(String),
    ToolUpdated(String),
    ToolRemoved(String),
    SourceAdded(String), // server_id
    SourceRemoved(String),
}

/// Metrics for registry operations
#[derive(Debug, Clone, Default)]
pub struct RegistryMetrics {
    pub total_tools: usize,
    pub local_tools: usize,
    pub federated_tools: usize,
    pub schema_validation_errors: u64,
    pub last_federation_sync: Option<DateTime<Utc>>,
    pub federation_failures: u64,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            change_listeners: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(RegistryMetrics::default())),
        }
    }

    /// Register a tool specification
    pub async fn register_tool(&self, tool: ToolSpec) -> Result<()> {
        let tool_name = tool.name.clone();
        let is_update = self.tools.contains_key(&tool_name);

        // Compute schema hash
        let hash = self.compute_tool_hash(&tool);
        let mut tool_with_hash = tool;
        tool_with_hash.spec_hash = hash;
        tool_with_hash.last_updated = Utc::now();

        // Store the tool
        let tool_arc = Arc::new(tool_with_hash);
        self.tools.insert(tool_name.clone(), tool_arc.clone());

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_tools = self.tools.len();
            metrics.local_tools = self
                .tools
                .iter()
                .filter(|entry| matches!(entry.value().source, ToolSource::Local))
                .count();
            metrics.federated_tools = metrics.total_tools - metrics.local_tools;
        }

        // Notify listeners
        let event = if is_update {
            RegistryChangeEvent::ToolUpdated(tool_name)
        } else {
            RegistryChangeEvent::ToolAdded(tool_name)
        };
        self.notify_listeners(event);

        Ok(())
    }

    /// Get a tool specification by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<ToolSpec>> {
        self.tools.get(name).map(|entry| entry.value().clone())
    }

    /// Get all tool specifications
    pub fn get_all_tools(&self) -> Vec<Arc<ToolSpec>> {
        self.tools
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get tools from a specific source
    pub fn get_tools_from_source(&self, server_id: &str) -> Vec<Arc<ToolSpec>> {
        self.tools
            .iter()
            .filter(|entry| match &entry.value().source {
                ToolSource::Federated { server_id: sid, .. } => sid == server_id,
                ToolSource::Local => server_id == "local",
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove a tool by name
    pub async fn remove_tool(&self, name: &str) -> Option<Arc<ToolSpec>> {
        if let Some((_, tool)) = self.tools.remove(name) {
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_tools = self.tools.len();
                metrics.local_tools = self
                    .tools
                    .iter()
                    .filter(|entry| matches!(entry.value().source, ToolSource::Local))
                    .count();
                metrics.federated_tools = metrics.total_tools - metrics.local_tools;
            }

            // Notify listeners
            self.notify_listeners(RegistryChangeEvent::ToolRemoved(name.to_string()));
            Some(tool)
        } else {
            None
        }
    }

    /// Remove all tools from a specific source
    pub async fn remove_tools_from_source(&self, server_id: &str) -> Vec<String> {
        let tools_to_remove: Vec<String> = self
            .tools
            .iter()
            .filter(|entry| match &entry.value().source {
                ToolSource::Federated { server_id: sid, .. } => sid == server_id,
                ToolSource::Local => server_id == "local",
            })
            .map(|entry| entry.key().clone())
            .collect();

        for tool_name in &tools_to_remove {
            self.tools.remove(tool_name);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_tools = self.tools.len();
            metrics.local_tools = self
                .tools
                .iter()
                .filter(|entry| matches!(entry.value().source, ToolSource::Local))
                .count();
            metrics.federated_tools = metrics.total_tools - metrics.local_tools;
        }

        // Notify listeners
        if !tools_to_remove.is_empty() {
            self.notify_listeners(RegistryChangeEvent::SourceRemoved(server_id.to_string()));
        }

        tools_to_remove
    }

    /// Validate tool arguments against schema
    pub async fn validate_tool_arguments(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<(), Vec<String>> {
        use jsonschema::JSONSchema;

        let tool = self.get_tool(tool_name).ok_or_else(|| {
            vec![format!("Tool '{}' not found in registry", tool_name)]
        })?;

        // Compile JSON schema
        let schema = JSONSchema::compile(&tool.input_schema).map_err(|e| {
            vec![format!("Invalid JSON schema for tool '{}': {}", tool_name, e)]
        })?;

        // Validate arguments
        let validation_result = schema.validate(arguments);
        if let Err(errors) = validation_result {
            let error_messages: Vec<String> = errors
                .into_iter()
                .map(|error| format!("{}", error))
                .collect();

            // Update error metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.schema_validation_errors += 1;
            }

            return Err(error_messages);
        }

        Ok(())
    }

    /// Generate MCP catalog resource
    pub async fn generate_catalog(&self) -> serde_json::Value {
        let tools: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|entry| {
                let tool = entry.value();
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema,
                    "outputSchema": tool.output_schema,
                    "source": tool.source,
                    "specVersion": tool.spec_version,
                    "specHash": tool.spec_hash,
                    "lastUpdated": tool.last_updated,
                    "metadata": tool.metadata
                })
            })
            .collect();

        let metrics = self.metrics.read().await;
        serde_json::json!({
            "catalog": {
                "version": "1.0",
                "generatedAt": Utc::now(),
                "tools": tools,
                "summary": {
                    "totalTools": metrics.total_tools,
                    "localTools": metrics.local_tools,
                    "federatedTools": metrics.federated_tools,
                    "lastFederationSync": metrics.last_federation_sync
                }
            }
        })
    }

    /// Add a change listener
    pub async fn add_change_listener(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<RegistryChangeEvent>,
    ) {
        let mut listeners = self.change_listeners.write().await;
        listeners.push(sender);
    }

    /// Get registry metrics
    pub async fn get_metrics(&self) -> RegistryMetrics {
        self.metrics.read().await.clone()
    }

    /// Compute SHA-256 hash of tool specifications
    fn compute_tool_hash(&self, tool: &ToolSpec) -> String {
        let mut hasher = Sha256::new();
        
        // Hash the core schema components
        if let Ok(schema_bytes) = serde_json::to_vec(&tool.input_schema) {
            hasher.update(&schema_bytes);
        }
        
        if let Some(ref output_schema) = tool.output_schema {
            if let Ok(output_bytes) = serde_json::to_vec(output_schema) {
                hasher.update(&output_bytes);
            }
        }
        
        hasher.update(tool.name.as_bytes());
        hasher.update(tool.description.as_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    /// Notify all change listeners
    fn notify_listeners(&self, event: RegistryChangeEvent) {
        let rt = tokio::runtime::Handle::current();
        let self_clone = self.clone();
        rt.spawn(async move {
            let listeners = self_clone.change_listeners.read().await;
            let mut dead_listeners = Vec::new();

            for (index, sender) in listeners.iter().enumerate() {
                if sender.send(event.clone()).is_err() {
                    dead_listeners.push(index);
                }
            }

            // Remove dead listeners
            if !dead_listeners.is_empty() {
                drop(listeners);
                let mut listeners = self_clone.change_listeners.write().await;
                for &index in dead_listeners.iter().rev() {
                    listeners.remove(index);
                }
            }
        });
    }

    /// Initialize with local tools (synchronous version)
    pub fn seed_with_local_tools(&self) -> Result<()> {
        // Define the built-in local tools
        let local_tools = vec![
            ToolSpec {
                name: "web_search_exa".to_string(),
                description: "Search the web with consciousness-aware context injection".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "numResults": {"type": "number"},
                        "projectPath": {"type": "string", "description": "Project path for context discovery"},
                        "perceptionIds": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["query"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {"type": "string"},
                        "results": {"type": "array"},
                        "context_enhanced": {"type": "boolean"},
                        "consciousness_coordination": {"type": "object"}
                    }
                })),
                source: ToolSource::Local,
                spec_version: "1.0.0".to_string(),
                spec_hash: String::new(), // Will be computed
                last_updated: Utc::now(),
                metadata: serde_json::json!({"category": "search", "consciousness_aware": true}),
            },
            ToolSpec {
                name: "deep_researcher_start".to_string(),
                description: "Start comprehensive research with paradox-resilient methodology".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "instructions": {"type": "string"},
                        "model": {"type": "string", "enum": ["exa-research", "exa-research-pro"]},
                        "projectPath": {"type": "string"},
                        "paradoxTolerance": {"type": "number", "minimum": 0, "maximum": 1}
                    },
                    "required": ["instructions"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {"type": "string"},
                        "status": {"type": "string"},
                        "consciousness_enhanced": {"type": "boolean"}
                    }
                })),
                source: ToolSource::Local,
                spec_version: "1.0.0".to_string(),
                spec_hash: String::new(),
                last_updated: Utc::now(),
                metadata: serde_json::json!({"category": "research", "consciousness_aware": true}),
            },
            // Add more built-in tools...
        ];

        // Register all local tools synchronously using the blocking method
        for tool in local_tools {
            if let Err(e) = self.register_tool_sync(tool) {
                return Err(e);
            }
        }

        tracing::info!("Seeded registry with {} local tools", self.tools.len());
        Ok(())
    }

    /// Synchronous tool registration for initialization
    fn register_tool_sync(&self, mut tool: ToolSpec) -> Result<()> {
        // Generate hash if not provided
        if tool.spec_hash.is_empty() {
            tool.spec_hash = self.compute_tool_hash(&tool);
        }
        tool.last_updated = Utc::now();

        // Insert into registry
        let tool_name = tool.name.clone();
        self.tools.insert(tool_name.clone(), Arc::new(tool));

        // Log registration for debugging
        tracing::debug!("Tool registered synchronously: {}", tool_name);
        
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.get_all_tools().len(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let registry = ToolRegistry::new();
        
        let tool = ToolSpec {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}),
            output_schema: None,
            source: ToolSource::Local,
            spec_version: "1.0.0".to_string(),
            spec_hash: String::new(),
            last_updated: Utc::now(),
            metadata: serde_json::json!({}),
        };

        registry.register_tool(tool).unwrap();
        
        let retrieved = registry.get_tool("test_tool").unwrap();
        assert_eq!(retrieved.name, "test_tool");
        assert!(!retrieved.spec_hash.is_empty());
    }

    #[test]
    fn test_tool_validation() {
        let registry = ToolRegistry::new();
        
        let tool = ToolSpec {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
            output_schema: None,
            source: ToolSource::Local,
            spec_version: "1.0.0".to_string(),
            spec_hash: String::new(),
            last_updated: Utc::now(),
            metadata: serde_json::json!({}),
        };

        registry.register_tool(tool).unwrap();

        // Valid arguments
        let valid_args = serde_json::json!({"query": "test query"});
        assert!(registry.validate_tool_arguments("test_tool", &valid_args).is_ok());

        // Invalid arguments (missing required field)
        let invalid_args = serde_json::json!({"other_field": "value"});
        assert!(registry.validate_tool_arguments("test_tool", &invalid_args).is_err());
    }

    #[test]
    fn test_catalog_generation() {
        let registry = ToolRegistry::new();
        registry.seed_with_local_tools().unwrap();
        
        let catalog = registry.generate_catalog();
        assert!(catalog["catalog"]["tools"].is_array());
        assert!(catalog["catalog"]["summary"]["totalTools"].as_u64().unwrap() > 0);
    }
}