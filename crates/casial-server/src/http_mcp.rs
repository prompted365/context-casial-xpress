//! HTTP/SSE MCP endpoint implementation for Smithery integration
//!
//! Provides MCP protocol support over HTTP with Server-Sent Events (SSE)
//! to enable registration and operation with Smithery.ai and other MCP clients.

use anyhow::Result;
use axum::{
    extract::State,
    http::{header, Method, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
    body::Body,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use crate::{mcp::*, AppState};


/// MCP HTTP handler - supports both POST for JSON-RPC and GET for SSE
pub async fn mcp_handler(
    method: Method,
    State(state): State<AppState>,
    body: Option<String>,
) -> Result<Response, StatusCode> {
    match method {
        Method::POST => handle_post(state, body).await,
        Method::GET => handle_get_sse(state).await,
        Method::HEAD => {
            // Return OK for HEAD requests (used by Smithery for health checks)
            Ok(StatusCode::OK.into_response())
        }
        Method::OPTIONS => {
            // Handle CORS preflight
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, MCP-Protocol-Version")
                .body(axum::body::Body::empty())
                .unwrap())
        }
        _ => Ok(StatusCode::METHOD_NOT_ALLOWED.into_response()),
    }
}

/// Handle POST requests with JSON-RPC payloads
async fn handle_post(
    state: AppState,
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
        "tools/call" => handle_tool_call(&state, request).await,
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

    Ok(Json(response).into_response())
}

/// Handle GET requests for SSE stream
async fn handle_get_sse(
    _state: AppState,
) -> Result<Response, StatusCode> {
    // For Smithery's Streamable HTTP, we need to return a simple SSE stream
    // that will handle JSON-RPC messages sent as events
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    
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
    
    Ok(response
        .into_response())
}

/// Handle initialize request
async fn handle_initialize(
    state: &AppState,
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
            "name": "casial-server",
            "title": "Context-Casial-Xpress MCP Server",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "A consciousness-aware context coordination server for AI systems. Part of Ubiquity OS - where paradoxes make the system stronger."
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

    // For now, tool execution is not implemented
    // This would need to be integrated with the actual tool execution logic
    warn!("Tool execution not yet implemented for HTTP transport: {}", params.name);
    create_success_response(request.id, json!({
        "content": [{
            "type": "text", 
            "text": "Tool execution is not yet implemented for HTTP transport. Please use WebSocket transport for full functionality."
        }],
        "isError": true
    }))
}

/// Handle completion request
async fn handle_completion(
    state: &AppState,
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
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let config = json!({
        "name": "context-casial-xpress",
        "title": "Context-Casial-Xpress MCP Server",
        "description": "A consciousness-aware context coordination server for AI systems",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": ["streamable-http"],
        "configSchema": {
            "type": "object",
            "properties": {
                "debug": {
                    "type": "boolean",
                    "description": "Enable debug logging",
                    "default": false
                },
                "consciousness_mode": {
                    "type": "string",
                    "description": "Consciousness integration mode",
                    "enum": ["full", "partial", "disabled"],
                    "default": "full"
                },
                "max_context_size": {
                    "type": "integer",
                    "description": "Maximum context size in characters",
                    "minimum": 1000,
                    "maximum": 1000000,
                    "default": 100000
                }
            }
        }
    });

    Ok(Json(config))
}