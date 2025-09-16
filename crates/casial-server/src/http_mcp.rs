//! HTTP/SSE MCP endpoint implementation for Smithery integration
//!
//! Provides MCP protocol support over HTTP with Server-Sent Events (SSE)
//! to enable registration and operation with Smithery.ai and other MCP clients.

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{header, Method, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use http::HeaderValue;

use crate::{mcp::*, AppState};

/// Session configuration from query parameters
#[derive(Debug, Default, Deserialize)]
pub struct SessionConfig {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub debug: Option<bool>,
    pub consciousness_mode: Option<String>,
    pub max_context_size: Option<i32>,
    pub agent_role: Option<String>,
    pub mission: Option<String>,
    pub shim_enabled: Option<bool>,
}

/// Query parameters that may include base64 encoded config
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    #[serde(flatten)]
    pub direct_params: SessionConfig,
    pub config: Option<String>, // Base64-encoded JSON config
}


/// MCP HTTP handler - supports both POST for JSON-RPC and GET for SSE
pub async fn mcp_handler(
    method: Method,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
    body: Option<String>,
) -> Result<Response, StatusCode> {
    // Extract config from base64 if provided, otherwise use direct params
    let config = if let Some(encoded_config) = params.config {
        // Decode base64 config like Python implementation
        match BASE64.decode(&encoded_config) {
            Ok(decoded) => {
                match serde_json::from_slice::<SessionConfig>(&decoded) {
                    Ok(parsed_config) => {
                        debug!("Decoded config from base64: {:?}", parsed_config);
                        parsed_config
                    }
                    Err(e) => {
                        warn!("Failed to parse base64 config JSON: {}", e);
                        params.direct_params
                    }
                }
            }
            Err(e) => {
                warn!("Failed to decode base64 config: {}", e);
                params.direct_params
            }
        }
    } else {
        params.direct_params
    };
    // Validate API key
    const VALID_API_KEY: &str = "GiftFromUbiquityF2025";
    
    if let Some(ref api_key) = config.api_key {
        if api_key != VALID_API_KEY {
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(header::CONTENT_TYPE, "application/json")
                .body(
                    Json(json!({
                        "error": "Invalid API key",
                        "message": "Please provide a valid API key in the configuration"
                    }))
                    .into_response()
                    .into_body(),
                )
                .unwrap());
        }
    } else {
        // API key is required
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(
                Json(json!({
                    "error": "Missing API key",
                    "message": "API key is required. Please configure with apiKey parameter."
                }))
                .into_response()
                .into_body(),
            )
            .unwrap());
    }
    
    // Log session configuration if debug is enabled
    if config.debug.unwrap_or(false) {
        info!("Session config: consciousness_mode={:?}, max_context_size={:?}", 
            config.consciousness_mode, config.max_context_size);
    }
    
    match method {
        Method::POST => handle_post(state, config, body).await,
        Method::GET => handle_get_sse(state, config).await,
        Method::HEAD => {
            // Return OK for HEAD requests (used by Smithery for health checks)
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, MCP-Protocol-Version")
                .header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")
                .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "mcp-session-id, mcp-protocol-version")
                .body(axum::body::Body::empty())
                .unwrap())
        }
        Method::OPTIONS => {
            // Handle CORS preflight with proper headers for Smithery
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, MCP-Protocol-Version")
                .header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")
                .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "mcp-session-id, mcp-protocol-version")
                .body(axum::body::Body::empty())
                .unwrap())
        }
        _ => Ok(StatusCode::METHOD_NOT_ALLOWED.into_response()),
    }
}

/// Handle POST requests with JSON-RPC payloads
async fn handle_post(
    state: AppState,
    config: SessionConfig,
    body: Option<String>,
) -> Result<Response, StatusCode> {
    let body = body.ok_or(StatusCode::BAD_REQUEST)?;
    
    // Parse JSON-RPC request
    let request: JsonRpcRequest = serde_json::from_str(&body)
        .map_err(|e| {
            error!("Failed to parse JSON-RPC request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    debug!("Received MCP request: method={}, id={:?}", request.method, request.id);

    // Route to appropriate handler
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, request).await,
        "notifications/initialized" => handle_initialized(&state, request).await,
        "tools/list" => handle_tools_list(&state, request).await,
        "tools/call" => handle_tool_call(&state, request, config.agent_role.as_deref()).await,
        "completion/complete" => handle_completion(&state, request).await,
        _ => {
            warn!("Unknown MCP method: {}", request.method);
            create_error_response(
                request.id,
                -32601,
                "Method not found",
                Some(json!({ "method": request.method })),
            )
        }
    };

    // Create the response with CORS headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, HEAD, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, MCP-Protocol-Version")
        .header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")
        .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "mcp-session-id, mcp-protocol-version")
        .body(Json(response).into_response().into_body())
        .unwrap();

    Ok(response)
}

/// Handle GET requests for SSE stream
async fn handle_get_sse(
    _state: AppState,
    _config: SessionConfig,
) -> Result<Response, StatusCode> {
    // For Smithery's Streamable HTTP, we need to return a simple SSE stream
    // that will handle JSON-RPC messages sent as events
    let (_tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    
    // Don't send any initial events - let the client initiate
    // This matches the Streamable HTTP specification
    
    // Convert receiver to stream
    let stream = ReceiverStream::new(rx);
    
    // Set up SSE response with appropriate headers
    let response = Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(30))
                .text(":\n"),  // Standard SSE keep-alive format
        );
    
    // Add CORS headers to SSE response
    let mut sse_response = response.into_response();
    let headers = sse_response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, HEAD, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Content-Type, Authorization, MCP-Protocol-Version"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static("mcp-session-id, mcp-protocol-version"));
    
    Ok(sse_response)
}

/// Handle initialize request
async fn handle_initialize(
    _state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    // Extract initialize params
    #[derive(Deserialize)]
    struct InitializeParams {
        #[serde(rename = "protocolVersion")]
        protocol_version: String,
        capabilities: Value,
        #[serde(rename = "clientInfo")]
        client_info: Option<Value>,
    }

    let params: InitializeParams = match serde_json::from_value(request.params) {
        Ok(p) => p,
        Err(e) => {
            return create_error_response(
                request.id,
                -32602,
                "Invalid params",
                Some(json!({ "error": e.to_string() })),
            );
        }
    };

    info!("MCP initialize: protocol_version={}, client_info={:?}", 
        params.protocol_version, params.client_info);

    // Check protocol version compatibility
    let supported_version = "2024-11-05";
    let negotiated_version = if params.protocol_version == supported_version {
        supported_version
    } else {
        // For now, we only support one version
        warn!("Client requested unsupported protocol version: {}", params.protocol_version);
        supported_version
    };

    // Build server capabilities
    let server_capabilities = json!({
        "tools": {
            "listChanged": true
        },
        "logging": {},
        "completion": {
            "enabled": true
        },
        "experimental": {
            "consciousness": true,
            "paradox_handling": true
        }
    });

    // Build response
    let result = json!({
        "protocolVersion": negotiated_version,
        "capabilities": server_capabilities,
        "serverInfo": {
            "name": "mop-server",
            "title": "Meta-Orchestration Protocol (MOP) Server",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "Meta-Orchestration Protocol (MOP): An MCP orchestration framework that acts as a consciousness-aware proxy layer. Use 'orchestrate_mcp_proxy' to augment any MCP server's tools with context injection, swarm instructions, and paradox handling. Use 'discover_mcp_tools' to analyze and map tools from other servers. Part of Ubiquity OS - where paradoxes make the system stronger."
    });

    create_success_response(request.id, result)
}

/// Handle initialized notification
async fn handle_initialized(
    _state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    info!("MCP client initialized");
    
    // This is a notification, so we don't send a response
    // But since we're in HTTP mode, we'll send an empty success
    create_success_response(request.id, json!({}))
}

/// Handle tools/list request
async fn handle_tools_list(
    state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    info!("Listing MCP tools");

    // Get tools from registry
    let tools = state.tool_registry.get_all_tools();
    
    // Convert to MCP tool format
    let mcp_tools: Vec<Value> = tools.into_iter().map(|tool| {
        json!({
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.input_schema
        })
    }).collect();

    let result = json!({
        "tools": mcp_tools
    });

    create_success_response(request.id, result)
}

/// Handle tools/call request
async fn handle_tool_call(
    state: &AppState,
    request: JsonRpcRequest,
    agent_role: Option<&str>,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct ToolCallParams {
        name: String,
        arguments: Option<Value>,
    }

    let params: ToolCallParams = match serde_json::from_value(request.params) {
        Ok(p) => p,
        Err(e) => {
            return create_error_response(
                request.id,
                -32602,
                "Invalid params",
                Some(json!({ "error": e.to_string() })),
            );
        }
    };

    info!("Calling tool: {}", params.name);

    // Apply pitfall avoidance shim to augment the request
    let augmented_args = {
        let shim = state.pitfall_shim.read().await;
        let args = params.arguments.unwrap_or(json!({}));
        match shim.augment_request(&params.name, &args, agent_role) {
            Ok(augmented) => augmented,
            Err(e) => {
                warn!("Failed to augment request with shim: {}", e);
                args
            }
        }
    };

    // For now, tool execution is not implemented for HTTP transport
    // This would need to be integrated with the actual tool execution logic
    warn!("Tool execution not yet implemented for HTTP transport: {}", params.name);
    
    // Create a mock response showing the augmented request
    let mock_response = json!({
        "tool_called": params.name,
        "augmented_arguments": augmented_args,
        "message": "Tool execution is not yet implemented for HTTP transport. The augmented arguments show how the pitfall avoidance shim would enhance the request.",
        "shim_applied": true
    });

    // Process the response through the shim
    let processed_response = {
        let shim = state.pitfall_shim.read().await;
        match shim.process_response(&params.name, &mock_response) {
            Ok(processed) => processed,
            Err(e) => {
                warn!("Failed to process response with shim: {}", e);
                mock_response
            }
        }
    };

    create_success_response(request.id, json!({
        "content": [{
            "type": "text", 
            "text": serde_json::to_string_pretty(&processed_response).unwrap_or_default()
        }],
        "isError": false
    }))
}

/// Handle completion request
async fn handle_completion(
    _state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct CompletionParams {
        #[serde(rename = "ref")]
        reference: CompletionReference,
        argument: CompletionArgument,
    }

    #[derive(Deserialize)]
    struct CompletionReference {
        #[serde(rename = "type")]
        ref_type: String,
        name: String,
    }

    #[derive(Deserialize)]
    struct CompletionArgument {
        name: String,
        value: String,
    }

    let _params: CompletionParams = match serde_json::from_value(request.params) {
        Ok(p) => p,
        Err(e) => {
            return create_error_response(
                request.id,
                -32602,
                "Invalid params",
                Some(json!({ "error": e.to_string() })),
            );
        }
    };

    // For now, return empty completions
    // This can be enhanced later with actual completion logic
    let result = json!({
        "completion": {
            "values": [],
            "hasMore": false,
            "total": 0
        }
    });

    create_success_response(request.id, result)
}

/// Well-known configuration endpoint handler
pub async fn well_known_config_handler(
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let config = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "$id": "https://meta-orchestration-protocol-production.up.railway.app/.well-known/mcp-config",
        "title": "MCP Session Configuration",
        "description": "Configuration for Meta-Orchestration Protocol (MOP) server. This server acts as a consciousness-aware proxy that can augment and coordinate tool calls across multiple MCP servers.",
        "x-query-style": "dot+bracket",
        "type": "object",
        "properties": {
            "apiKey": {
                "type": "string",
                "title": "API Key",
                "description": "Your API key for authentication (use 'GiftFromUbiquityF2025' for access)",
                "default": "GiftFromUbiquityF2025"
            },
            "agent_role": {
                "type": "string",
                "title": "Agent Role",
                "description": "Role of the calling agent for context-aware responses",
                "enum": ["researcher", "analyst", "monitor", "watcher", "orchestrator"],
                "default": "orchestrator"
            },
            "consciousness_mode": {
                "type": "string",
                "title": "Consciousness Mode",
                "description": "Level of consciousness integration",
                "enum": ["full", "partial", "disabled"],
                "default": "full"
            },
            "max_context_size": {
                "type": "integer",
                "title": "Max Context Size",
                "description": "Maximum context size in characters",
                "minimum": 1000,
                "maximum": 1000000,
                "default": 100000
            },
            "mission": {
                "type": "string",
                "title": "Mission Profile",
                "description": "Pre-configured mission to load",
                "enum": ["exa-orchestration", "general", "research", "monitoring"],
                "default": "exa-orchestration"
            },
            "shim_enabled": {
                "type": "boolean",
                "title": "Enable Pitfall Avoidance Shim",
                "description": "Enable automatic context injection and pitfall warnings",
                "default": true
            },
            "debug": {
                "type": "boolean",
                "title": "Debug Mode",
                "description": "Enable debug logging",
                "default": false
            }
        },
        "required": ["apiKey"],
        "additionalProperties": false,
        
        // Additional metadata for Smithery
        "name": "meta-orchestration-protocol",
        "title": "Meta-Orchestration Protocol (MOP) Server",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": ["streamable-http"]
    });

    Ok(Json(config))
}