//! # WebSocket Handler
//!
//! High-performance WebSocket communication for consciousness-aware context coordination.

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{mcp, AppState};
use casial_core::{CoordinationRequest, PerceptionId};

/// WebSocket session information
#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub session_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub message_count: usize,
    pub active_coordination_id: Option<Uuid>,
    pub active_perceptions: Vec<PerceptionId>,
}

impl WebSocketSession {
    fn new() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            created_at: Utc::now(),
            message_count: 0,
            active_coordination_id: None,
            active_perceptions: Vec::new(),
        }
    }
}

/// WebSocket handler for MCP communication
pub struct WebSocketHandler {
    state: AppState,
}

impl WebSocketHandler {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(self, socket: WebSocket) {
        let session = WebSocketSession::new();
        let session_id = session.session_id;

        info!("🔌 New WebSocket connection: {}", session_id);

        // Register session
        self.state.active_sessions.insert(session_id, session);

        // Split socket for concurrent read/write
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Create bounded channel for backpressure control
        let (app_sender, mut app_receiver) = tokio::sync::mpsc::channel::<String>(64);

        // Create heartbeat channels
        let (heartbeat_sender, mut heartbeat_receiver) =
            tokio::sync::mpsc::unbounded_channel::<()>();

        // Spawn writer task with backpressure handling
        let writer_task = tokio::spawn(async move {
            let mut heartbeat_interval =
                tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                tokio::select! {
                    // Handle outgoing messages with backpressure
                    msg = app_receiver.recv() => {
                        match msg {
                            Some(message) => {
                                if let Err(e) = ws_sender.send(Message::Text(message)).await {
                                    tracing::error!("Failed to send WebSocket message: {}", e);
                                    break;
                                }
                            }
                            None => {
                                tracing::debug!("Message channel closed, ending writer task");
                                break;
                            }
                        }
                    }

                    // Send periodic heartbeat pings
                    _ = heartbeat_interval.tick() => {
                        if let Err(e) = ws_sender.send(Message::Ping(vec![])).await {
                            tracing::error!("Failed to send heartbeat ping: {}", e);
                            break;
                        }
                        tracing::trace!("Sent heartbeat ping to session {}", session_id);
                    }

                    // Handle heartbeat responses (pongs)
                    _ = heartbeat_receiver.recv() => {
                        tracing::trace!("Received heartbeat pong from session {}", session_id);
                        // Reset heartbeat timeout if needed
                    }
                }
            }
        });

        // Message handling loop with sender channel
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("📨 Received message: {}", text);

                    match self.handle_text_message(&text, session_id).await {
                        Ok(Some(response)) => {
                            // Use bounded channel with backpressure
                            match app_sender.try_send(response) {
                                Ok(()) => {}
                                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                                    error!("WebSocket send buffer full for session {}, dropping message", session_id);
                                    let error_response = mcp::create_error_response(
                                        serde_json::Value::Null,
                                        -32603,
                                        "Server busy - send buffer full",
                                        Some(serde_json::json!({"reason": "backpressure"})),
                                    );
                                    // Try to send error, but don't block
                                    let _ = app_sender
                                        .try_send(serde_json::to_string(&error_response).unwrap());
                                }
                                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                    error!(
                                        "WebSocket send channel closed for session {}",
                                        session_id
                                    );
                                    break;
                                }
                            }
                        }
                        Ok(None) => {
                            // No response needed
                        }
                        Err(e) => {
                            error!("Error handling message: {}", e);
                            let error_response = mcp::create_error_response(
                                serde_json::Value::Null,
                                -32603,
                                "Internal error",
                                Some(serde_json::json!({"error": e.to_string()})),
                            );

                            let error_json = serde_json::to_string(&error_response).unwrap();
                            if app_sender.try_send(error_json).is_err() {
                                error!("Failed to send error response for session {}", session_id);
                                break;
                            }
                        }
                    }

                    // Update message count
                    if let Some(mut session) = self.state.active_sessions.get_mut(&session_id) {
                        session.message_count += 1;
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary message (not supported)");
                }
                Ok(Message::Ping(ping)) => {
                    debug!("Received ping, sending pong");
                    // Send pong through the writer channel as binary message
                    let _pong_msg = Message::Pong(ping);
                    // For pongs, we bypass the text channel and handle directly
                    // This is a limitation - we'd need a more complex channel system for full support
                    tracing::trace!("Received ping from session {}", session_id);
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from client");
                    // Notify heartbeat system that we received a pong
                    let _ = heartbeat_sender.send(());
                }
                Ok(Message::Close(_)) => {
                    info!("🔌 WebSocket connection closed by client: {}", session_id);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // Clean up session and writer task
        drop(app_sender); // Close sender to signal writer task to end
        let _ = writer_task.await; // Wait for writer task to complete

        self.state.active_sessions.remove(&session_id);
        info!("🔌 WebSocket connection ended: {}", session_id);
    }

    /// Handle text messages (JSON-RPC)
    async fn handle_text_message(&self, text: &str, session_id: Uuid) -> Result<Option<String>> {
        // Parse JSON-RPC request
        let request: mcp::JsonRpcRequest = serde_json::from_str(text)?;

        debug!("🔧 Processing JSON-RPC method: {}", request.method);

        // Handle different MCP methods
        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await?,
            "tools/list" => self.handle_tools_list(request).await?,
            "tools/call" => self.handle_tools_call(request, session_id).await?,
            "resources/list" => self.handle_resources_list(request).await?,
            "resources/read" => self.handle_resources_read(request).await?,
            "casial/debug" => self.handle_casial_debug(request, session_id).await?,
            "casial/perception/add" => self.handle_add_perception(request, session_id).await?,
            "casial/perception/remove" => {
                self.handle_remove_perception(request, session_id).await?
            }
            _ => mcp::create_error_response(
                request.id,
                -32601,
                "Method not found",
                Some(serde_json::json!({"method": request.method})),
            ),
        };

        Ok(Some(serde_json::to_string(&response)?))
    }

    /// Handle MCP initialize method
    async fn handle_initialize(
        &self,
        request: mcp::JsonRpcRequest,
    ) -> Result<mcp::JsonRpcResponse> {
        info!("🤝 MCP initialization requested");

        let server_info = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": true
                },
                "casial": {
                    "consciousness_aware": true,
                    "paradox_handling": true,
                    "perception_coordination": true,
                    "substrate_integration": true
                }
            },
            "serverInfo": {
                "name": "meta-orchestration-protocol",
                "version": env!("CARGO_PKG_VERSION"),
                "part_of": "ubiquity-os",
                "consciousness_substrate": "active",
                "hydraulic_lime_principle": "stronger_under_pressure"
            }
        });

        Ok(mcp::create_success_response(request.id, server_info))
    }

    /// Handle tools/list method
    async fn handle_tools_list(
        &self,
        request: mcp::JsonRpcRequest,
    ) -> Result<mcp::JsonRpcResponse> {
        debug!("🔧 Listing available tools from registry");

        // Get all tools from registry (local + federated)
        let all_tools = self.state.tool_registry.get_all_tools();

        let tools_json: Vec<serde_json::Value> = all_tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema,
                    "outputSchema": tool.output_schema
                })
            })
            .collect();

        let response = serde_json::json!({
            "tools": tools_json
        });

        Ok(mcp::create_success_response(request.id, response))
    }

    /// Handle resources/list method
    async fn handle_resources_list(
        &self,
        request: mcp::JsonRpcRequest,
    ) -> Result<mcp::JsonRpcResponse> {
        debug!("📋 Listing available resources");

        let resources = serde_json::json!({
            "resources": [
                {
                    "uri": "mcp://catalog",
                    "name": "Tool Catalog",
                    "description": "Federated tool specifications and metadata",
                    "mimeType": "application/json"
                }
            ]
        });

        Ok(mcp::create_success_response(request.id, resources))
    }

    /// Handle resources/read method
    async fn handle_resources_read(
        &self,
        request: mcp::JsonRpcRequest,
    ) -> Result<mcp::JsonRpcResponse> {
        debug!("📖 Reading resource");

        let uri = request
            .params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing URI parameter"))?;

        match uri {
            "mcp://catalog" => {
                let catalog = self.state.tool_registry.generate_catalog().await;
                Ok(mcp::create_success_response(request.id, catalog))
            }
            _ => Ok(mcp::create_error_response(
                request.id,
                -32601,
                "Resource not found",
                Some(serde_json::json!({"uri": uri})),
            )),
        }
    }

    /// Process a "tools/call" JSON-RPC request by validating arguments, attempting federation routing,
    /// and falling back to local consciousness-aware coordination and tool execution.
    ///
    /// The response contains the tool execution result and a `consciousness_coordination` object
    /// describing coordination details (applied flag, injected content, activated rules, used templates,
    /// paradoxes detected and handling, and metadata). If federation routing succeeds, its result is
    /// returned; validation failures produce a JSON-RPC invalid-parameters error.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a JSON-RPC request for calling a tool named "example_tool".
    /// let request = mcp::JsonRpcRequest {
    ///     id: Some(serde_json::json!(1)),
    ///     method: "tools/call".to_string(),
    ///     params: serde_json::json!({
    ///         "name": "example_tool",
    ///         "arguments": { "foo": "bar" },
    ///         "mode": "execute"
    ///     }),
    /// };
    ///
    /// // `handler` is an instance of WebSocketHandler available in the surrounding context.
    /// // The call is async and returns a `mcp::JsonRpcResponse`.
    /// // let response = tokio::runtime::Handle::current().block_on(handler.handle_tools_call(request, session_id)).unwrap();
    /// ```
    async fn handle_tools_call(
        &self,
        request: mcp::JsonRpcRequest,
        session_id: Uuid,
    ) -> Result<mcp::JsonRpcResponse> {
        let params = request.params;
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let mut args = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Extract execution mode (request-level)
        let mode = params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("execute");

        if let Some(obj) = args.as_object_mut() {
            obj.remove("mode");
        }

        info!(
            "🔧 Executing tool: {} with consciousness coordination (mode: {})",
            tool_name, mode
        );

        // Validate tool arguments against schema
        if let Err(validation_errors) = self
            .state
            .tool_registry
            .validate_tool_arguments(tool_name, &args)
            .await
        {
            return Ok(mcp::create_error_response(
                request.id,
                -32602,
                "Invalid parameters",
                Some(serde_json::json!({
                    "validation_errors": validation_errors
                })),
            ));
        }

        // Try federation routing first
        let federation_result = {
            let federation_guard = self.state.federation_manager.read().await;
            if let Some(federation_manager) = federation_guard.as_ref() {
                use crate::federation::ExecutionMode;

                let execution_mode = match mode {
                    "plan" => ExecutionMode::Plan,
                    "hybrid" => ExecutionMode::Hybrid,
                    _ => ExecutionMode::Execute,
                };

                Some(
                    federation_manager
                        .route_tool_call(tool_name, args.clone(), execution_mode)
                        .await,
                )
            } else {
                None
            }
        };

        if let Some(result) = federation_result {
            match result {
                Ok(result) => {
                    let response_content = serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result)?
                        }]
                    });
                    return Ok(mcp::create_success_response(request.id, response_content));
                }
                Err(e) => {
                    warn!(
                        "Federation routing failed, falling back to local execution: {}",
                        e
                    );
                }
            }
        }

        // Fallback to local execution with consciousness coordination
        let active_perceptions = self
            .state
            .active_sessions
            .get(&session_id)
            .map(|s| s.active_perceptions.clone())
            .unwrap_or_default();

        let project_path = args
            .get("projectPath")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let paradox_tolerance = args
            .get("paradoxTolerance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        let environment = std::env::vars().collect();

        let coordination_request = CoordinationRequest {
            tool_name: tool_name.to_string(),
            tool_args: args.clone(),
            environment,
            project_path,
            active_perceptions,
            paradox_tolerance,
        };

        let coordination_result = {
            let engine = self.state.casial_engine.write().await;
            engine.coordinate(coordination_request)?
        };

        if let Some(mut session) = self.state.active_sessions.get_mut(&session_id) {
            session.active_coordination_id = Some(Uuid::new_v4());
        }

        let tool_result = self
            .execute_tool(tool_name, &coordination_result.modified_args)
            .await?;

        let response_content = serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&serde_json::json!({
                    "tool_execution": tool_result,
                    "consciousness_coordination": {
                        "applied": coordination_result.applied,
                        "injected_content": coordination_result.injected_content,
                        "activated_rules": coordination_result.activated_rules,
                        "used_templates": coordination_result.used_templates,
                        "paradoxes_detected": coordination_result.paradoxes_detected.len(),
                        "paradox_handling": coordination_result.paradoxes_detected.iter().map(|p| {
                            serde_json::json!({
                                "id": p.id,
                                "description": p.description,
                                "strategy": format!("{:?}", p.resolution_strategy)
                            })
                        }).collect::<Vec<_>>(),
                        "metadata": coordination_result.metadata
                    }
                }))?
            }]
        });

        Ok(mcp::create_success_response(request.id, response_content))
    }

    /// Execute tool with coordinated context (simulated)
    async fn execute_tool(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // This simulates tool execution with the context-modified arguments
        // In a real implementation, this would call actual external APIs

        match tool_name {
            "web_search_exa" => Ok(serde_json::json!({
                "status": "success",
                "tool": "web_search_exa",
                "query": args.get("query").unwrap_or(&serde_json::Value::Null),
                "results_count": args.get("numResults").and_then(|n| n.as_u64()).unwrap_or(5),
                "context_enhanced": true,
                "simulation": true
            })),
            "deep_researcher_start" => Ok(serde_json::json!({
                "status": "success",
                "tool": "deep_researcher_start",
                "instructions": args.get("instructions").unwrap_or(&serde_json::Value::Null),
                "model": args.get("model").unwrap_or(&serde_json::json!("exa-research")),
                "task_id": Uuid::new_v4(),
                "consciousness_enhanced": true,
                "simulation": true
            })),
            "crawling_exa" => Ok(serde_json::json!({
                "status": "success",
                "tool": "crawling_exa",
                "url": args.get("url").unwrap_or(&serde_json::Value::Null),
                "max_chars": args.get("maxCharacters").and_then(|n| n.as_u64()).unwrap_or(3000),
                "context_aware": true,
                "simulation": true
            })),
            "linkedin_search_exa" => Ok(serde_json::json!({
                "status": "success",
                "tool": "linkedin_search_exa",
                "query": args.get("query").unwrap_or(&serde_json::Value::Null),
                "searchType": args.get("searchType").unwrap_or(&serde_json::json!("all")),
                "results_count": args.get("numResults").and_then(|n| n.as_u64()).unwrap_or(5),
                "context_enhanced": true,
                "professional_network_focus": true,
                "sample_profile": "breyden-taylor",
                "simulation": true
            })),
            "company_research_exa" => Ok(serde_json::json!({
                "status": "success",
                "tool": "company_research_exa",
                "companyName": args.get("companyName").unwrap_or(&serde_json::Value::Null),
                "results_count": args.get("numResults").and_then(|n| n.as_u64()).unwrap_or(5),
                "context_enhanced": true,
                "mission_driven": true,
                "research_depth": "comprehensive",
                "simulation": true
            })),
            _ => Ok(serde_json::json!({
                "status": "success",
                "tool": tool_name,
                "args": args,
                "consciousness_coordinated": true,
                "simulation": true
            })),
        }
    }

    /// Handle Casial debug method
    async fn handle_casial_debug(
        &self,
        request: mcp::JsonRpcRequest,
        session_id: Uuid,
    ) -> Result<mcp::JsonRpcResponse> {
        let params = request.params;
        let tool_name = params
            .get("toolName")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing toolName"))?;

        let args = params.get("args").cloned().unwrap_or(serde_json::json!({}));

        let show_paradoxes = params
            .get("showParadoxes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let _show_perceptions = params
            .get("showPerceptions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        debug!("🔍 Debug request for tool: {}", tool_name);

        // Get session info
        let session_info = self
            .state
            .active_sessions
            .get(&session_id)
            .map(|s| {
                serde_json::json!({
                    "session_id": s.session_id,
                    "message_count": s.message_count,
                    "active_perceptions": s.active_perceptions.len()
                })
            })
            .unwrap_or(serde_json::json!({}));

        // Get engine statistics
        let engine_stats = {
            let engine = self.state.casial_engine.read().await;
            let coordination_history = engine.get_coordination_history();
            let paradox_registry = engine.get_paradox_registry();

            serde_json::json!({
                "total_coordinations": coordination_history.len(),
                "total_paradoxes": paradox_registry.len(),
                "paradoxes": if show_paradoxes {
                    paradox_registry.iter().map(|p| serde_json::json!({
                        "id": p.id,
                        "description": p.description,
                        "strategy": format!("{:?}", p.resolution_strategy),
                        "confidence_impact": p.confidence_impact,
                        "conflicting_perceptions": p.conflicting_perceptions.len()
                    })).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            })
        };

        let debug_info = serde_json::json!({
            "debug_request": {
                "tool_name": tool_name,
                "args": args
            },
            "session": session_info,
            "casial_engine": engine_stats,
            "consciousness_substrate": {
                "active": true,
                "hydraulic_lime_principle": "stronger_under_pressure",
                "paradox_resilience": "operational"
            }
        });

        Ok(mcp::create_success_response(request.id, debug_info))
    }

    /// Handle adding perception to session
    async fn handle_add_perception(
        &self,
        request: mcp::JsonRpcRequest,
        session_id: Uuid,
    ) -> Result<mcp::JsonRpcResponse> {
        let params = request.params;
        let perception_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing perception name"))?;

        let perception_id = PerceptionId::new();

        // Add to session
        if let Some(mut session) = self.state.active_sessions.get_mut(&session_id) {
            session.active_perceptions.push(perception_id);
        }

        info!(
            "👁️ Added perception '{}' to session {}",
            perception_name, session_id
        );

        let response = serde_json::json!({
            "perception_id": perception_id,
            "name": perception_name,
            "session_id": session_id,
            "active_perceptions": self.state.active_sessions
                .get(&session_id)
                .map(|s| s.active_perceptions.len())
                .unwrap_or(0)
        });

        Ok(mcp::create_success_response(request.id, response))
    }

    /// Handle removing perception from session
    async fn handle_remove_perception(
        &self,
        request: mcp::JsonRpcRequest,
        session_id: Uuid,
    ) -> Result<mcp::JsonRpcResponse> {
        let params = request.params;
        let perception_id_str = params
            .get("perception_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing perception_id"))?;

        // Parse perception ID (simplified - in real implementation would parse from UUID string)
        let target_perception = PerceptionId::new(); // Placeholder

        // Remove from session
        let removed = if let Some(mut session) = self.state.active_sessions.get_mut(&session_id) {
            let initial_len = session.active_perceptions.len();
            session
                .active_perceptions
                .retain(|&id| id != target_perception);
            initial_len > session.active_perceptions.len()
        } else {
            false
        };

        let response = serde_json::json!({
            "removed": removed,
            "perception_id": perception_id_str,
            "session_id": session_id,
            "remaining_perceptions": self.state.active_sessions
                .get(&session_id)
                .map(|s| s.active_perceptions.len())
                .unwrap_or(0)
        });

        Ok(mcp::create_success_response(request.id, response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::ServerConfig, pitfall_shim::PitfallAvoidanceShim};

    #[test]
    fn test_websocket_session_creation() {
        let session = WebSocketSession::new();
        assert_eq!(session.message_count, 0);
        assert!(session.active_coordination_id.is_none());
        assert_eq!(session.active_perceptions.len(), 0);
    }

    #[tokio::test]
    async fn test_websocket_handler_creation() {
        let config = ServerConfig::default();
        let shim = PitfallAvoidanceShim::default();
        let state = AppState::new(config, shim);
        let handler = WebSocketHandler::new(state);

        // Handler should be created successfully
        assert_eq!(handler.state.active_sessions.len(), 0);
    }
}
