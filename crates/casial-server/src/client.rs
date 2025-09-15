//! # MCP Downstream Client
//!
//! WebSocket JSON-RPC client for connecting to downstream MCP servers.

use crate::{config::DownstreamMcpServer, mcp};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Connection state for downstream MCP server
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Health metrics for downstream connection
#[derive(Debug, Clone, Default)]
pub struct ConnectionHealth {
    pub state: ConnectionState,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub message_count: u64,
    pub error_count: u64,
    pub latency_ms: f64,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Pending request tracking
struct PendingRequest {
    sender: oneshot::Sender<Result<mcp::JsonRpcResponse>>,
    sent_at: DateTime<Utc>,
    timeout: Duration,
}

/// MCP client for downstream server communication
pub struct McpClient {
    config: DownstreamMcpServer,
    health: Arc<RwLock<ConnectionHealth>>,
    sender: Option<mpsc::UnboundedSender<ClientCommand>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug)]
enum ClientCommand {
    Send {
        request: mcp::JsonRpcRequest,
        response_tx: oneshot::Sender<Result<mcp::JsonRpcResponse>>,
    },
    Disconnect,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(config: DownstreamMcpServer) -> Self {
        Self {
            config,
            health: Arc::new(RwLock::new(ConnectionHealth::default())),
            sender: None,
            handle: None,
        }
    }

    /// Connect to the downstream MCP server
    pub async fn connect(&mut self) -> Result<()> {
        if self.is_connected().await {
            return Ok(());
        }

        info!("🔗 Connecting to downstream MCP server: {}", self.config.name);

        // Update state
        {
            let mut health = self.health.write().await;
            health.state = ConnectionState::Connecting;
        }

        // Create command channel
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        self.sender = Some(cmd_tx);

        // Spawn connection task
        let config = self.config.clone();
        let health = Arc::clone(&self.health);
        
        self.handle = Some(tokio::spawn(async move {
            if let Err(e) = Self::connection_task(config, health, cmd_rx).await {
                error!("Connection task failed: {}", e);
            }
        }));

        // Wait for connection to establish
        let mut attempts = 0;
        let max_attempts = 30; // 3 seconds with 100ms intervals
        
        while attempts < max_attempts {
            if self.is_connected().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        Err(anyhow::anyhow!("Failed to establish connection within timeout"))
    }

    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        matches!(self.health.read().await.state, ConnectionState::Connected)
    }

    /// Get connection health
    pub async fn get_health(&self) -> ConnectionHealth {
        self.health.read().await.clone()
    }

    /// Send MCP initialize request
    pub async fn initialize(&self) -> Result<mcp::JsonRpcResponse> {
        let request = mcp::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::String(Uuid::new_v4().to_string()),
            method: "initialize".to_string(),
            params: serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "clientInfo": {
                    "name": "context-casial-xpress-proxy",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        };

        self.send_request(request).await
    }

    /// List available tools from downstream server
    pub async fn list_tools(&self) -> Result<mcp::JsonRpcResponse> {
        let request = mcp::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::String(Uuid::new_v4().to_string()),
            method: "tools/list".to_string(),
            params: Value::Object(serde_json::Map::new()),
        };

        self.send_request(request).await
    }

    /// Call a tool on downstream server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<mcp::JsonRpcResponse> {
        let request = mcp::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::String(Uuid::new_v4().to_string()),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": name,
                "arguments": arguments
            }),
        };

        self.send_request(request).await
    }

    /// List resources from downstream server
    pub async fn list_resources(&self) -> Result<mcp::JsonRpcResponse> {
        let request = mcp::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::String(Uuid::new_v4().to_string()),
            method: "resources/list".to_string(),
            params: Value::Object(serde_json::Map::new()),
        };

        self.send_request(request).await
    }

    /// Read a resource from downstream server
    pub async fn read_resource(&self, uri: &str) -> Result<mcp::JsonRpcResponse> {
        let request = mcp::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::String(Uuid::new_v4().to_string()),
            method: "resources/read".to_string(),
            params: serde_json::json!({
                "uri": uri
            }),
        };

        self.send_request(request).await
    }

    /// Send a request to the downstream server
    async fn send_request(&self, request: mcp::JsonRpcRequest) -> Result<mcp::JsonRpcResponse> {
        let sender = self.sender.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        let (response_tx, response_rx) = oneshot::channel();
        
        sender.send(ClientCommand::Send {
            request,
            response_tx,
        }).context("Failed to send command")?;

        response_rx.await.context("Response channel closed")?
    }

    /// Main connection task
    async fn connection_task(
        config: DownstreamMcpServer,
        health: Arc<RwLock<ConnectionHealth>>,
        mut cmd_rx: mpsc::UnboundedReceiver<ClientCommand>,
    ) -> Result<()> {
        let url = config.url.clone();
        
        // Establish WebSocket connection
        let (ws_stream, _) = connect_async(&url).await
            .context("Failed to connect to downstream MCP server")?;

        info!("✅ Connected to downstream MCP server: {}", config.name);

        // Update health
        {
            let mut health = health.write().await;
            health.state = ConnectionState::Connected;
            health.connected_at = Some(Utc::now());
        }

        // Split stream
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let mut pending_requests = std::collections::HashMap::<String, PendingRequest>::new();

        // Heartbeat task
        let health_clone = Arc::clone(&health);
        let heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut health = health_clone.write().await;
                health.last_heartbeat = Some(Utc::now());
            }
        });

        // Main message loop
        loop {
            tokio::select! {
                // Handle incoming commands
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(ClientCommand::Send { request, response_tx }) => {
                            let request_id = request.id.clone();
                            let request_json = serde_json::to_string(&request)?;
                            
                            // Store pending request
                            if let Value::String(id) = &request_id {
                                pending_requests.insert(id.clone(), PendingRequest {
                                    sender: response_tx,
                                    sent_at: Utc::now(),
                                    timeout: Duration::from_millis(config.timeout_ms),
                                });
                            }

                            // Send request
                            if let Err(e) = ws_sender.send(Message::Text(request_json)).await {
                                error!("Failed to send WebSocket message: {}", e);
                                break;
                            }

                            // Update metrics
                            {
                                let mut health = health.write().await;
                                health.message_count += 1;
                            }
                        }
                        Some(ClientCommand::Disconnect) => {
                            info!("🔌 Disconnecting from downstream MCP server: {}", config.name);
                            break;
                        }
                        None => break,
                    }
                }

                // Handle incoming messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            debug!("📨 Received from {}: {}", config.name, text);

                            match serde_json::from_str::<mcp::JsonRpcResponse>(&text) {
                                Ok(response) => {
                                    // Find pending request
                                    if let Value::String(id) = &response.id {
                                        if let Some(pending) = pending_requests.remove(id) {
                                            // Calculate latency
                                            let latency = Utc::now().signed_duration_since(pending.sent_at);
                                            {
                                                let mut health = health.write().await;
                                                health.latency_ms = latency.num_milliseconds() as f64;
                                            }

                                            // Send response
                                            let _ = pending.sender.send(Ok(response));
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse JSON-RPC response: {}", e);
                                    let mut health = health.write().await;
                                    health.error_count += 1;
                                }
                            }
                        }
                        Some(Ok(Message::Binary(_))) => {
                            // Ignore binary messages for now
                        }
                        Some(Ok(Message::Ping(data))) => {
                            // Send pong response
                            if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Pong received - connection is alive
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("🔌 WebSocket closed by downstream server: {}", config.name);
                            break;
                        }
                        Some(Ok(Message::Frame(_))) => {
                            // Raw frames - ignore
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error from {}: {}", config.name, e);
                            let mut health = health.write().await;
                            health.error_count += 1;
                            break;
                        }
                        None => break,
                    }
                }

                // Timeout cleanup
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    let now = Utc::now();
                    let mut timed_out = Vec::new();

                    for (id, pending) in &pending_requests {
                        if now.signed_duration_since(pending.sent_at).to_std().unwrap_or_default() > pending.timeout {
                            timed_out.push(id.clone());
                        }
                    }

                    for id in timed_out {
                        if let Some(pending) = pending_requests.remove(&id) {
                            let _ = pending.sender.send(Err(anyhow::anyhow!("Request timeout")));
                            let mut health = health.write().await;
                            health.error_count += 1;
                        }
                    }
                }
            }
        }

        heartbeat_task.abort();

        // Update health to disconnected
        {
            let mut health = health.write().await;
            health.state = ConnectionState::Disconnected;
        }

        // Fail all pending requests
        for (_, pending) in pending_requests {
            let _ = pending.sender.send(Err(anyhow::anyhow!("Connection closed")));
        }

        Ok(())
    }

    /// Disconnect from downstream server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(sender) = &self.sender {
            let _ = sender.send(ClientCommand::Disconnect);
        }

        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }

        self.sender = None;
        
        {
            let mut health = self.health.write().await;
            health.state = ConnectionState::Disconnected;
        }

        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = DownstreamMcpServer {
            id: "test".to_string(),
            name: "Test Server".to_string(),
            url: "ws://localhost:8001/ws".to_string(),
            connection_type: "websocket".to_string(),
            enabled: true,
            timeout_ms: 5000,
            priority: 1,
            auth: None,
        };

        let client = McpClient::new(config);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_health_tracking() {
        let config = DownstreamMcpServer {
            id: "test".to_string(),
            name: "Test Server".to_string(),
            url: "ws://localhost:8001/ws".to_string(),
            connection_type: "websocket".to_string(),
            enabled: true,
            timeout_ms: 5000,
            priority: 1,
            auth: None,
        };

        let client = McpClient::new(config);
        let health = client.get_health();
        
        assert!(matches!(health.state, ConnectionState::Disconnected));
        assert_eq!(health.message_count, 0);
        assert_eq!(health.error_count, 0);
    }
}