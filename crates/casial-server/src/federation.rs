//! # MCP Federation Manager
//!
//! Manages federation of multiple downstream MCP servers, tool aggregation, and intelligent routing.

use crate::{
    client::McpClient,
    config::FederationSettings,
    registry::{ToolRegistry, ToolSource, ToolSpec},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::{mapref::entry::Entry, DashMap};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Federation metrics and status
#[derive(Debug, Clone, Default)]
pub struct FederationMetrics {
    pub active_connections: usize,
    pub total_servers: usize,
    pub tool_calls_forwarded: u64,
    pub federation_errors: u64,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_duration_ms: f64,
    pub server_failures: HashMap<String, u64>,
    pub open_circuits: usize,
    pub circuit_open_skips: u64,
}

/// Execution mode for tool calls
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Execute,
    Plan,
    Hybrid,
}

/// Execution plan for deferred execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub target_server: String,
    pub created_at: DateTime<Utc>,
    pub estimated_cost: Option<f64>,
    pub dependencies: Vec<String>,
    pub spec_ref: Option<String>,
}

/// MCP Federation Manager
pub struct McpFederationManager {
    settings: FederationSettings,
    clients: Arc<DashMap<String, Arc<RwLock<McpClient>>>>,
    tool_registry: Arc<ToolRegistry>,
    metrics: Arc<RwLock<FederationMetrics>>,
    notification_sender: Arc<RwLock<Option<mpsc::UnboundedSender<FederationEvent>>>>,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
    failure_tracker: Arc<DashMap<String, CircuitState>>,
    tool_cache: Arc<DashMap<String, ToolCacheEntry>>,
}

/// Federation events for notifications
#[derive(Debug, Clone)]
pub enum FederationEvent {
    ServerConnected(String),
    ServerDisconnected(String),
    ToolsUpdated(String, usize),
    SyncCompleted,
    Error(String),
}

#[derive(Debug, Clone)]
struct ToolCacheEntry {
    spec_hash: String,
    expires_at: Instant,
    tool_count: usize,
}

#[derive(Debug, Clone)]
struct CircuitState {
    failure_count: u32,
    last_failure: Option<Instant>,
    open_until: Option<Instant>,
    reset_after: Duration,
}

impl CircuitState {
    fn new(reset_seconds: u64) -> Self {
        Self {
            failure_count: 0,
            last_failure: None,
            open_until: None,
            reset_after: Duration::from_secs(reset_seconds.max(1)),
        }
    }

    fn is_open(&mut self, now: Instant) -> bool {
        if let Some(until) = self.open_until {
            if now < until {
                return true;
            }
            self.open_until = None;
            self.failure_count = 0;
        }

        if let Some(last) = self.last_failure {
            if now.duration_since(last) >= self.reset_after {
                self.failure_count = 0;
                self.last_failure = None;
            }
        }

        false
    }

    fn is_open_now(&self, now: Instant) -> bool {
        self.open_until.map(|until| now < until).unwrap_or(false)
    }

    fn register_failure(
        &mut self,
        now: Instant,
        settings: &FederationSettings,
    ) -> Option<Duration> {
        if let Some(last) = self.last_failure {
            if now.duration_since(last) >= self.reset_after {
                self.failure_count = 0;
            }
        }

        self.failure_count = self.failure_count.saturating_add(1);
        self.last_failure = Some(now);

        let threshold = settings.circuit_breaker_threshold.max(1);

        if self.failure_count >= threshold {
            let penalty_attempt = self.failure_count - threshold;
            let duration = compute_backoff_duration(settings, penalty_attempt);
            self.open_until = Some(now + duration);
            Some(duration)
        } else {
            None
        }
    }

    fn register_success(&mut self) {
        self.failure_count = 0;
        self.open_until = None;
        self.last_failure = None;
    }
}

fn compute_backoff_duration(settings: &FederationSettings, attempt: u32) -> Duration {
    let base = settings.backoff_initial_ms.max(10);
    let max_backoff = settings.backoff_max_ms.max(base);
    let power = attempt.min(16);
    let multiplier = 1u64.checked_shl(power.into()).unwrap_or(u64::MAX);
    let mut backoff_ms = base.saturating_mul(multiplier);
    if backoff_ms > max_backoff {
        backoff_ms = max_backoff;
    }
    let jitter_max = base.min(max_backoff);
    let jitter = rand::thread_rng().gen_range(0..=jitter_max);
    let total_ms = backoff_ms.saturating_add(jitter).max(1);
    Duration::from_millis(total_ms)
}

async fn record_failure_shared(
    failure_tracker: &DashMap<String, CircuitState>,
    metrics: &Arc<RwLock<FederationMetrics>>,
    server_id: &str,
    settings: &FederationSettings,
    error_message: &str,
) -> Option<Duration> {
    let now = Instant::now();

    let open_duration = {
        match failure_tracker.entry(server_id.to_string()) {
            Entry::Occupied(mut entry) => entry.get_mut().register_failure(now, settings),
            Entry::Vacant(entry) => {
                let mut state = CircuitState::new(settings.circuit_breaker_reset_seconds);
                let duration = state.register_failure(now, settings);
                entry.insert(state);
                duration
            }
        }
    };

    let open_circuits = failure_tracker
        .iter()
        .filter(|entry| entry.value().is_open_now(now))
        .count();

    {
        let mut metrics_guard = metrics.write().await;
        metrics_guard.federation_errors += 1;
        *metrics_guard
            .server_failures
            .entry(server_id.to_string())
            .or_insert(0) += 1;
        metrics_guard.open_circuits = open_circuits;
    }

    if let Some(duration) = open_duration {
        warn!(
            "Circuit opened for {} for {:?} after failure: {}",
            server_id, duration, error_message
        );
    } else {
        warn!("Failure recorded for {}: {}", server_id, error_message);
    }

    open_duration
}

async fn record_success_shared(
    failure_tracker: &DashMap<String, CircuitState>,
    metrics: &Arc<RwLock<FederationMetrics>>,
    server_id: &str,
) {
    {
        if let Some(mut entry) = failure_tracker.get_mut(server_id) {
            entry.value_mut().register_success();
        }
    }

    let now = Instant::now();
    let open_circuits = failure_tracker
        .iter()
        .filter(|entry| entry.value().is_open_now(now))
        .count();

    let mut metrics_guard = metrics.write().await;
    metrics_guard.open_circuits = open_circuits;
}

impl McpFederationManager {
    /// Create a new federation manager
    pub fn new(settings: FederationSettings, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            settings,
            clients: Arc::new(DashMap::new()),
            tool_registry,
            metrics: Arc::new(RwLock::new(FederationMetrics::default())),
            notification_sender: Arc::new(RwLock::new(None)),
            sync_handle: None,
            failure_tracker: Arc::new(DashMap::new()),
            tool_cache: Arc::new(DashMap::new()),
        }
    }

    /// Initialize federation with downstream servers
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.settings.enabled {
            info!("🚫 MCP Federation disabled in configuration");
            return Ok(());
        }

        info!(
            "🌐 Initializing MCP Federation with {} downstream servers",
            self.settings.downstream_servers.len()
        );

        // Create clients for each downstream server
        for server_config in &self.settings.downstream_servers {
            if !server_config.enabled {
                continue;
            }

            info!(
                "🔧 Setting up downstream MCP server: {}",
                server_config.name
            );
            let client = Arc::new(RwLock::new(McpClient::new(server_config.clone())));
            self.clients.insert(server_config.id.clone(), client);
        }

        // Start periodic sync task
        if self.settings.catalog_refresh_interval > 0 {
            self.start_sync_task().await?;
        }

        // Perform initial sync
        self.sync_all_servers().await?;

        info!("✅ MCP Federation initialized successfully");
        Ok(())
    }

    /// Connect to all enabled downstream servers
    pub async fn connect_all(&self) -> Result<()> {
        let mut connection_tasks = Vec::new();

        for client_entry in self.clients.iter() {
            let server_id = client_entry.key().clone();
            let client = Arc::clone(client_entry.value());

            let task = tokio::spawn(async move {
                let result = {
                    let mut client = client.write().await;
                    client.connect().await
                };
                match result {
                    Ok(()) => {
                        info!("✅ Connected to downstream server: {}", server_id);
                        Ok(server_id)
                    }
                    Err(e) => {
                        error!("❌ Failed to connect to {}: {}", server_id, e);
                        Err(e)
                    }
                }
            });

            connection_tasks.push(task);
        }

        // Wait for all connections
        let results = futures::future::join_all(connection_tasks).await;
        let mut successful_connections = 0;

        for result in results {
            match result {
                Ok(Ok(_)) => successful_connections += 1,
                Ok(Err(e)) => error!("Connection error: {}", e),
                Err(e) => error!("Task error: {}", e),
            }
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.active_connections = successful_connections;
            metrics.total_servers = self.clients.len();
        }

        if successful_connections > 0 {
            info!(
                "🌐 Connected to {}/{} downstream servers",
                successful_connections,
                self.clients.len()
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to connect to any downstream servers"
            ))
        }
    }

    /// Sync tools from all connected servers
    pub async fn sync_all_servers(&self) -> Result<()> {
        let sync_start = std::time::Instant::now();
        info!("🔄 Starting federation sync...");

        let mut sync_tasks = Vec::new();

        let settings = self.settings.clone();
        let tool_cache = Arc::clone(&self.tool_cache);
        let failure_tracker = Arc::clone(&self.failure_tracker);
        let metrics = Arc::clone(&self.metrics);

        for client_entry in self.clients.iter() {
            let server_id = client_entry.key().clone();
            let client = Arc::clone(client_entry.value());
            let registry = Arc::clone(&self.tool_registry);
            let tool_cache = Arc::clone(&tool_cache);
            let failure_tracker = Arc::clone(&failure_tracker);
            let metrics = Arc::clone(&metrics);
            let settings = settings.clone();

            let task = tokio::spawn(async move {
                Self::sync_server_tools(
                    server_id,
                    client,
                    registry,
                    tool_cache,
                    metrics,
                    failure_tracker,
                    settings,
                )
                .await
            });

            sync_tasks.push(task);
        }

        // Wait for all sync tasks
        let results = futures::future::join_all(sync_tasks).await;
        let mut total_tools = 0;
        let mut errors = 0;

        for result in results {
            match result {
                Ok(Ok(tool_count)) => total_tools += tool_count,
                Ok(Err(e)) => {
                    error!("Sync error: {}", e);
                    errors += 1;
                }
                Err(e) => {
                    error!("Task error: {}", e);
                    errors += 1;
                }
            }
        }

        let sync_duration = sync_start.elapsed();

        {
            let mut metrics = self.metrics.write().await;
            metrics.last_sync = Some(Utc::now());
            metrics.sync_duration_ms = sync_duration.as_secs_f64() * 1000.0;
            metrics.federation_errors += errors;
        }

        info!(
            "✅ Federation sync completed: {} tools from {} servers ({:.2}ms)",
            total_tools,
            self.clients.len(),
            sync_duration.as_secs_f64() * 1000.0
        );

        Ok(())
    }

    /// Sync tools from a specific server
    async fn sync_server_tools(
        server_id: String,
        client: Arc<RwLock<McpClient>>,
        registry: Arc<ToolRegistry>,
        tool_cache: Arc<DashMap<String, ToolCacheEntry>>,
        metrics: Arc<RwLock<FederationMetrics>>,
        failure_tracker: Arc<DashMap<String, CircuitState>>,
        settings: FederationSettings,
    ) -> Result<usize> {
        debug!("🔄 Syncing tools from server: {}", server_id);

        let now = Instant::now();

        let mut skip_due_to_circuit = false;
        {
            if let Some(mut state) = failure_tracker.get_mut(&server_id) {
                if state.is_open(now) {
                    debug!(
                        "Skipping sync for {} because circuit breaker is open",
                        server_id
                    );
                    skip_due_to_circuit = true;
                }
            }
        }

        if skip_due_to_circuit {
            {
                let mut metrics_guard = metrics.write().await;
                metrics_guard.circuit_open_skips =
                    metrics_guard.circuit_open_skips.saturating_add(1);
            }
            return Ok(0);
        }

        // Initialize client and get tools response
        let tools_response = {
            let client_guard = client.read().await;
            if !client_guard.is_connected().await {
                let message = format!("Server {} is not connected", server_id);
                record_failure_shared(&failure_tracker, &metrics, &server_id, &settings, &message)
                    .await;
                return Err(anyhow::anyhow!(message));
            }

            match client_guard.initialize().await {
                Ok(_) => debug!("✅ Initialized connection to {}", server_id),
                Err(e) => warn!("⚠️ Failed to initialize {}: {}", server_id, e),
            }

            client_guard
                .list_tools()
                .await
                .with_context(|| format!("Failed to list tools from server {}", server_id))
        }?;

        if let Some(error) = tools_response.error {
            let message = format!("Server {} returned error: {}", server_id, error.message);
            record_failure_shared(&failure_tracker, &metrics, &server_id, &settings, &message)
                .await;
            return Err(anyhow::anyhow!(message));
        }

        let tools_data = tools_response
            .result
            .ok_or_else(|| anyhow::anyhow!("No tools data from server {}", server_id))?;

        let tools = tools_data
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid tools format from server {}", server_id))?;

        let spec_hash = {
            let serialized = serde_json::to_vec(tools).map_err(|e| {
                anyhow::anyhow!("Failed to serialize tools from {}: {}", server_id, e)
            })?;
            let mut hasher = Sha256::new();
            hasher.update(serialized);
            format!("{:x}", hasher.finalize())
        };

        if settings.tool_cache_ttl_seconds > 0 {
            if let Some(mut cache_entry) = tool_cache.get_mut(&server_id) {
                if cache_entry.spec_hash == spec_hash && cache_entry.expires_at > now {
                    cache_entry.expires_at =
                        now + Duration::from_secs(settings.tool_cache_ttl_seconds.max(1));
                    debug!(
                        "Cache hit for {} ({} tools) – skipping registry update",
                        server_id, cache_entry.tool_count
                    );
                    record_success_shared(&failure_tracker, &metrics, &server_id).await;
                    return Ok(cache_entry.tool_count);
                }
            }
        }

        registry.remove_tools_from_source(&server_id).await;

        let mut registered_count = 0;
        for tool_data in tools {
            if let Ok(tool_spec) = Self::parse_tool_spec(tool_data, &server_id) {
                if registry.register_tool(tool_spec).await.is_ok() {
                    registered_count += 1;
                }
            }
        }

        if settings.tool_cache_ttl_seconds > 0 {
            tool_cache.insert(
                server_id.clone(),
                ToolCacheEntry {
                    spec_hash,
                    expires_at: Instant::now()
                        + Duration::from_secs(settings.tool_cache_ttl_seconds.max(1)),
                    tool_count: registered_count,
                },
            );
        }

        record_success_shared(&failure_tracker, &metrics, &server_id).await;

        info!(
            "📦 Registered {} tools from server: {}",
            registered_count, server_id
        );
        Ok(registered_count)
    }

    /// Parse tool specification from JSON
    fn parse_tool_spec(tool_data: &serde_json::Value, server_id: &str) -> Result<ToolSpec> {
        let name = tool_data
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let description = tool_data
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let input_schema = tool_data
            .get("inputSchema")
            .cloned()
            .unwrap_or(serde_json::json!({"type": "object"}));

        let output_schema = tool_data.get("outputSchema").cloned();

        // Get server URL for federated source
        let server_url = format!("server://{}", server_id); // Placeholder

        Ok(ToolSpec {
            name: name.to_string(),
            description,
            input_schema,
            output_schema,
            source: ToolSource::Federated {
                server_id: server_id.to_string(),
                server_url,
            },
            spec_version: "1.0.0".to_string(),
            spec_hash: String::new(), // Will be computed by registry
            last_updated: Utc::now(),
            metadata: tool_data
                .get("metadata")
                .cloned()
                .unwrap_or(serde_json::json!({})),
        })
    }

    /// Route tool call to appropriate server
    pub async fn route_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
        mode: ExecutionMode,
    ) -> Result<serde_json::Value> {
        // Get tool specification from registry
        let tool = self
            .tool_registry
            .get_tool(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found in registry", tool_name))?;

        match mode {
            ExecutionMode::Plan => self.generate_execution_plan(tool, arguments).await,
            ExecutionMode::Execute => self.execute_tool_call(tool, arguments).await,
            ExecutionMode::Hybrid => {
                // Generate plan and execute immediately
                let plan_result = self
                    .generate_execution_plan(tool.clone(), arguments.clone())
                    .await?;
                let execute_result = self.execute_tool_call(tool, arguments).await?;

                Ok(serde_json::json!({
                    "mode": "hybrid",
                    "plan": plan_result,
                    "execution": execute_result
                }))
            }
        }
    }

    /// Generate execution plan for a tool call
    async fn generate_execution_plan(
        &self,
        tool: Arc<ToolSpec>,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let plan = ExecutionPlan {
            plan_id: Uuid::new_v4().to_string(),
            tool_name: tool.name.clone(),
            arguments,
            target_server: match &tool.source {
                ToolSource::Local => "local".to_string(),
                ToolSource::Federated { server_id, .. } => server_id.clone(),
            },
            created_at: Utc::now(),
            estimated_cost: None,
            dependencies: vec![],
            spec_ref: Some(format!("mcp://catalog/tools/{}", tool.name)),
        };

        Ok(serde_json::to_value(plan)?)
    }

    /// Execute tool call
    async fn execute_tool_call(
        &self,
        tool: Arc<ToolSpec>,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match &tool.source {
            ToolSource::Local => {
                // Handle local tool execution (simulated for now)
                Ok(serde_json::json!({
                    "status": "success",
                    "tool": tool.name,
                    "result": "Local execution completed",
                    "source": "local"
                }))
            }
            ToolSource::Federated { server_id, .. } => {
                // Forward to downstream server
                self.forward_to_downstream(server_id, &tool.name, arguments)
                    .await
            }
        }
    }

    /// Forward tool call to downstream server
    async fn forward_to_downstream(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let client = self
            .clients
            .get(server_id)
            .ok_or_else(|| anyhow::anyhow!("Downstream server '{}' not found", server_id))?;

        let downstream = Arc::clone(client.value());
        drop(client);

        let now = Instant::now();
        let mut circuit_blocked = false;
        let mut retry_after = None;
        {
            if let Some(mut state) = self.failure_tracker.get_mut(server_id) {
                if state.is_open(now) {
                    retry_after = state
                        .open_until
                        .map(|until| until.saturating_duration_since(now));
                    circuit_blocked = true;
                }
            }
        }

        if circuit_blocked {
            {
                let mut metrics_guard = self.metrics.write().await;
                metrics_guard.circuit_open_skips =
                    metrics_guard.circuit_open_skips.saturating_add(1);
            }
            return Err(anyhow::anyhow!(format!(
                "Circuit open for server '{}' (retry in {:?})",
                server_id, retry_after
            )));
        }

        debug!(
            "🔀 Forwarding tool call '{}' to server: {}",
            tool_name, server_id
        );

        let max_attempts = std::cmp::max(1, self.settings.max_retries) as u32;
        let mut attempt = 0u32;
        let mut last_error: Option<anyhow::Error> = None;

        while attempt <= max_attempts {
            let call_result = {
                let client_guard = downstream.read().await;
                if !client_guard.is_connected().await {
                    let message = format!("Server '{}' is not connected", server_id);
                    record_failure_shared(
                        &self.failure_tracker,
                        &self.metrics,
                        server_id,
                        &self.settings,
                        &message,
                    )
                    .await;
                    return Err(anyhow::anyhow!(message));
                }
                client_guard.call_tool(tool_name, arguments.clone()).await
            };

            match call_result {
                Ok(response) => {
                    if let Some(error) = response.error {
                        let message = format!("Downstream error: {}", error.message);
                        let circuit_duration = record_failure_shared(
                            &self.failure_tracker,
                            &self.metrics,
                            server_id,
                            &self.settings,
                            &message,
                        )
                        .await;
                        last_error = Some(anyhow::anyhow!(message.clone()));

                        if let Some(duration) = circuit_duration {
                            return Err(anyhow::anyhow!(format!(
                                "Circuit opened for server '{}' ({:?}) after downstream error",
                                server_id, duration
                            )));
                        }
                    } else {
                        record_success_shared(&self.failure_tracker, &self.metrics, server_id)
                            .await;
                        {
                            let mut metrics = self.metrics.write().await;
                            metrics.tool_calls_forwarded += 1;
                        }
                        return Ok(response
                            .result
                            .unwrap_or(serde_json::json!({"status": "success"})));
                    }
                }
                Err(err) => {
                    let message = format!("Failed to forward to {}: {}", server_id, err);
                    let circuit_duration = record_failure_shared(
                        &self.failure_tracker,
                        &self.metrics,
                        server_id,
                        &self.settings,
                        &message,
                    )
                    .await;
                    last_error = Some(anyhow::anyhow!(message.clone()));

                    if let Some(duration) = circuit_duration {
                        return Err(anyhow::anyhow!(format!(
                            "Circuit opened for server '{}' ({:?}) after transport error",
                            server_id, duration
                        )));
                    }
                }
            }

            attempt = attempt.saturating_add(1);
            if attempt > max_attempts {
                break;
            }

            let backoff = compute_backoff_duration(&self.settings, attempt);
            tokio::time::sleep(backoff).await;
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!(format!(
                "Unknown downstream error while calling '{}' on {}",
                tool_name, server_id
            ))
        }))
    }

    /// Start periodic sync task
    async fn start_sync_task(&mut self) -> Result<()> {
        let interval = Duration::from_secs(self.settings.catalog_refresh_interval);
        let clients = Arc::clone(&self.clients);
        let registry = Arc::clone(&self.tool_registry);
        let metrics = Arc::clone(&self.metrics);
        let tool_cache = Arc::clone(&self.tool_cache);
        let failure_tracker = Arc::clone(&self.failure_tracker);
        let settings = self.settings.clone();

        let sync_task = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                debug!("🔄 Periodic federation sync starting...");

                // Sync all servers
                let sync_start = std::time::Instant::now();
                let mut total_tools = 0;
                let mut errors = 0;

                for client_entry in clients.iter() {
                    let server_id = client_entry.key().clone();
                    let client = Arc::clone(client_entry.value());

                    match Self::sync_server_tools(
                        server_id,
                        client,
                        Arc::clone(&registry),
                        Arc::clone(&tool_cache),
                        Arc::clone(&metrics),
                        Arc::clone(&failure_tracker),
                        settings.clone(),
                    )
                    .await
                    {
                        Ok(count) => total_tools += count,
                        Err(e) => {
                            error!("Periodic sync error: {}", e);
                            errors += 1;
                        }
                    }
                }

                let sync_duration = sync_start.elapsed();

                {
                    let mut metrics = metrics.write().await;
                    metrics.last_sync = Some(Utc::now());
                    metrics.sync_duration_ms = sync_duration.as_secs_f64() * 1000.0;
                    metrics.federation_errors += errors;
                }

                debug!(
                    "✅ Periodic sync completed: {} tools ({:.2}ms)",
                    total_tools,
                    sync_duration.as_secs_f64() * 1000.0
                );
            }
        });

        self.sync_handle = Some(sync_task);
        Ok(())
    }

    /// Get federation metrics
    pub async fn get_metrics(&self) -> FederationMetrics {
        self.metrics.read().await.clone()
    }

    /// Get connection health for all servers
    pub async fn get_connection_health(&self) -> HashMap<String, crate::client::ConnectionHealth> {
        let mut health_map = HashMap::new();

        for entry in self.clients.iter() {
            let server_id = entry.key().clone();
            let client = entry.value().read().await;
            health_map.insert(server_id, client.get_health().await);
        }

        health_map
    }

    /// Get list of active federated servers
    pub async fn get_active_servers(&self) -> Vec<serde_json::Value> {
        let mut servers = Vec::new();

        for entry in self.clients.iter() {
            let server_id = entry.key().clone();
            let client = entry.value().read().await;
            let is_connected = client.is_connected().await;

            // Find the config for this server
            let config = self
                .settings
                .downstream_servers
                .iter()
                .find(|s| s.id == server_id)
                .cloned();

            if let Some(cfg) = config {
                servers.push(serde_json::json!({
                    "id": server_id,
                    "name": cfg.name,
                    "transport": cfg.connection_type,
                    "connected": is_connected,
                    "enabled": cfg.enabled,
                    "tool_count": self.tool_registry.get_tools_from_source(&server_id).len()
                }));
            }
        }

        servers
    }

    /// Shutdown federation manager
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("🛑 Shutting down MCP Federation...");

        // Cancel sync task
        if let Some(handle) = self.sync_handle.take() {
            handle.abort();
        }

        // Disconnect all clients
        for client_entry in self.clients.iter() {
            let mut client = client_entry.value().write().await;
            if let Err(e) = client.disconnect().await {
                warn!("Error disconnecting from {}: {}", client_entry.key(), e);
            }
        }

        self.clients.clear();

        self.failure_tracker.clear();
        self.tool_cache.clear();

        {
            let mut metrics = self.metrics.write().await;
            metrics.active_connections = 0;
        }

        info!("✅ MCP Federation shutdown complete");
        Ok(())
    }
}

impl Drop for McpFederationManager {
    fn drop(&mut self) {
        if let Some(handle) = self.sync_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_federation_manager_creation() {
        let mut settings = FederationSettings::default();
        settings.enabled = true;

        let registry = Arc::new(ToolRegistry::new());
        let manager = McpFederationManager::new(settings, registry);

        assert_eq!(manager.clients.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_spec_parsing() {
        let tool_data = serde_json::json!({
            "name": "test_tool",
            "description": "A test tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }
        });

        let spec = McpFederationManager::parse_tool_spec(&tool_data, "test_server").unwrap();
        assert_eq!(spec.name, "test_tool");
        assert_eq!(spec.description, "A test tool");
        assert!(matches!(spec.source, ToolSource::Federated { .. }));
    }
}
