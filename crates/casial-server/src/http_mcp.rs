//! HTTP/SSE MCP endpoint implementation for Smithery integration
//!
//! Provides MCP protocol support over HTTP with Server-Sent Events (SSE)
//! to enable registration and operation with Smithery.ai and other MCP clients.

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use tower_http::cors::{Any, CorsLayer};

const ALLOWED_METHODS: &str = "GET, POST, DELETE, HEAD, OPTIONS";
const ALLOWED_HEADERS: &str =
    "Content-Type, Authorization, Accept, Cache-Control, Mcp-Session-Id, Mcp-Protocol-Version";
const EXPOSED_HEADERS: &str = "Mcp-Session-Id, Mcp-Protocol-Version";

/// Global CORS policy shared across manual responses
#[derive(Debug, Clone)]
pub struct CorsPolicy {
    origin_policy: OriginPolicy,
    allow_credentials: bool,
}

#[derive(Debug, Clone)]
enum OriginPolicy {
    Any,
    List(Vec<HeaderValue>),
}

impl CorsPolicy {
    fn from_env() -> Self {
        let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
        let allowed_origins = allowed_origins.trim();

        if allowed_origins.is_empty() {
            tracing::warn!(
                "ALLOWED_ORIGINS not set, allowing all origins without credentials (not recommended for production)"
            );
            return Self {
                origin_policy: OriginPolicy::Any,
                allow_credentials: false,
            };
        }

        if allowed_origins == "*" {
            tracing::info!("ALLOWED_ORIGINS='*', enabling wildcard CORS without credentials");
            return Self {
                origin_policy: OriginPolicy::Any,
                allow_credentials: false,
            };
        }

        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|origin| match origin.parse::<HeaderValue>() {
                Ok(value) => Some(value),
                Err(e) => {
                    tracing::error!("Failed to parse allowed origin '{}': {}", origin, e);
                    None
                }
            })
            .collect();

        if origins.is_empty() {
            tracing::warn!(
                "ALLOWED_ORIGINS parsed to empty list, falling back to wildcard without credentials"
            );
            Self {
                origin_policy: OriginPolicy::Any,
                allow_credentials: false,
            }
        } else {
            Self {
                origin_policy: OriginPolicy::List(origins),
                allow_credentials: true,
            }
        }
    }

    fn resolve_origin(&self, request_headers: &HeaderMap) -> Option<HeaderValue> {
        match &self.origin_policy {
            OriginPolicy::Any => Some(HeaderValue::from_static("*")),
            OriginPolicy::List(allowed) => {
                if let Some(request_origin) = request_headers
                    .get(header::ORIGIN)
                    .and_then(|value| value.to_str().ok())
                {
                    if let Some(matching) = allowed
                        .iter()
                        .find(|origin| origin.as_bytes() == request_origin.as_bytes())
                    {
                        return Some(matching.clone());
                    }
                }
                None
            }
        }
    }

    fn allow_credentials(&self) -> bool {
        self.allow_credentials
    }
}

static CORS_POLICY: Lazy<CorsPolicy> = Lazy::new(CorsPolicy::from_env);

/// Access the configured global CORS policy
pub fn cors_policy() -> &'static CorsPolicy {
    &CORS_POLICY
}

/// Build a [`CorsLayer`] that mirrors the manual headers emitted elsewhere
pub fn build_cors_layer() -> CorsLayer {
    let policy = cors_policy().clone();
    let allow_headers = vec![
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        header::CACHE_CONTROL,
        HeaderName::from_static("mcp-session-id"),
        HeaderName::from_static("mcp-protocol-version"),
    ];

    let methods = [
        Method::GET,
        Method::POST,
        Method::DELETE,
        Method::HEAD,
        Method::OPTIONS,
    ];

    match &policy.origin_policy {
        OriginPolicy::Any => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(allow_headers)
            .expose_headers([
                HeaderName::from_static("mcp-session-id"),
                HeaderName::from_static("mcp-protocol-version"),
            ]),
        OriginPolicy::List(origins) => {
            let mut layer = CorsLayer::new()
                .allow_origin(origins.clone())
                .allow_methods(methods)
                .allow_headers(allow_headers)
                .expose_headers([
                    HeaderName::from_static("mcp-session-id"),
                    HeaderName::from_static("mcp-protocol-version"),
                ]);

            if policy.allow_credentials() {
                layer = layer.allow_credentials(true);
            }

            layer
        }
    }
}

/// Apply manual CORS headers to a response
pub fn apply_cors_headers(headers: &mut HeaderMap, request_headers: &HeaderMap) {
    let policy = cors_policy();
    if let Some(origin) = policy.resolve_origin(request_headers) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

        if policy.allow_credentials() {
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        } else {
            headers.remove(header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
        }
    } else {
        headers.remove(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        headers.remove(header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
    }

    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static(ALLOWED_METHODS),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static(ALLOWED_HEADERS),
    );
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static(EXPOSED_HEADERS),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_env() {
        std::env::remove_var("ALLOWED_ORIGINS");
    }

    fn reset_sampling_flag() {
        std::env::remove_var("MOP_ENABLE_SAMPLING");
    }

    #[test]
    fn cors_policy_defaults_to_any_when_env_missing() {
        reset_env();
        let policy = CorsPolicy::from_env();
        let origin = policy.resolve_origin(&HeaderMap::new());

        assert_eq!(origin, Some(HeaderValue::from_static("*")));
        assert!(!policy.allow_credentials());
    }

    #[test]
    fn cors_policy_matches_listed_origin() {
        std::env::set_var("ALLOWED_ORIGINS", "https://example.com,https://other.test");
        let policy = CorsPolicy::from_env();

        let mut headers = HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://other.test"),
        );

        let origin = policy.resolve_origin(&headers);
        assert_eq!(origin, Some(HeaderValue::from_static("https://other.test")));
        assert!(policy.allow_credentials());
        reset_env();
    }

    #[test]
    fn cors_context_suppresses_credentials_for_wildcard() {
        std::env::set_var("ALLOWED_ORIGINS", "*");
        let policy = CorsPolicy::from_env();
        let origin = policy.resolve_origin(&HeaderMap::new());

        let mut headers = HeaderMap::new();
        apply_cors_headers(&mut headers, &HeaderMap::new());

        assert_eq!(origin, Some(HeaderValue::from_static("*")));
        assert!(!policy.allow_credentials());
        assert!(headers
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .is_none());
        reset_env();
    }

    #[test]
    fn sampling_disabled_by_default() {
        reset_sampling_flag();
        assert!(!super::sampling_feature_enabled());
    }

    #[test]
    fn sampling_enabled_for_truthy_values() {
        for value in ["true", "1", "yes"] {
            std::env::set_var("MOP_ENABLE_SAMPLING", value);
            assert!(
                super::sampling_feature_enabled(),
                "value {:?} should enable sampling",
                value
            );
        }
        reset_sampling_flag();
    }
}

use crate::{mcp::*, AppState};

/// Active session storage
#[derive(Debug, Clone)]
pub struct SessionData {
    pub id: String,
    pub config: SessionConfig,
    pub created_at: std::time::Instant,
    pub last_accessed: std::time::Instant,
}

/// Global session storage shared across requests
static SESSIONS: Lazy<Arc<DashMap<String, SessionData>>> = Lazy::new(|| Arc::new(DashMap::new()));

const DEMO_API_KEY: &str = "DEMO_KEY_PUBLIC";

static EXPECTED_API_KEY: Lazy<String> = Lazy::new(|| {
    let value = std::env::var("MOP_API_KEY").unwrap_or_else(|_| DEMO_API_KEY.to_string());
    if value == DEMO_API_KEY {
        tracing::info!(
            "Using public demo API key (DEMO KEY – public). Set MOP_API_KEY to override."
        );
    }
    value
});

fn expected_api_key() -> &'static str {
    EXPECTED_API_KEY.as_str()
}

fn sampling_feature_enabled() -> bool {
    std::env::var("MOP_ENABLE_SAMPLING")
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Session configuration from query parameters
#[derive(Debug, Default, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Default)]
pub struct QueryParams {
    #[serde(flatten)]
    pub direct_params: SessionConfig,
    pub config: Option<String>, // Base64-encoded JSON config
}

/// MCP HTTP handler - supports both POST for JSON-RPC and GET for SSE
pub async fn mcp_handler(
    method: Method,
    State(state): State<AppState>,
    headers: http::HeaderMap,
    Query(params): Query<QueryParams>,
    body: Option<String>,
) -> Result<Response, StatusCode> {
    // Extract config from base64 if provided, otherwise use direct params
    let mut config = if let Some(encoded_config) = params.config {
        // Decode base64 config like Python implementation
        match BASE64.decode(&encoded_config) {
            Ok(decoded) => match serde_json::from_slice::<SessionConfig>(&decoded) {
                Ok(parsed_config) => {
                    debug!("Decoded config from base64: {:?}", parsed_config);
                    parsed_config
                }
                Err(e) => {
                    warn!("Failed to parse base64 config JSON: {}", e);
                    params.direct_params
                }
            },
            Err(e) => {
                warn!("Failed to decode base64 config: {}", e);
                params.direct_params
            }
        }
    } else {
        params.direct_params
    };

    // Check for Bearer token authentication in headers (Smithery style)
    let mut api_key_from_header: Option<String> = None;
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some((scheme, token)) = auth_str.split_once(' ') {
                let token = token.trim();
                if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() {
                    api_key_from_header = Some(token.to_string());
                    debug!("Found Bearer token in Authorization header");
                }
            }
        }
    }

    // Use Bearer token if no API key in query params
    if config.api_key.is_none() && api_key_from_header.is_some() {
        config.api_key = api_key_from_header;
    }

    // Extract session ID from headers
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Check if we have a valid session (bypass API key check if so)
    let has_valid_session = if let Some(sid) = &session_id {
        SESSIONS.contains_key(sid)
    } else {
        false
    };

    // Validate API key only if no valid session
    if !has_valid_session {
        let expected_api_key = expected_api_key();

        if let Some(ref api_key) = config.api_key {
            if api_key != expected_api_key {
                let response = Response::builder()
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
                    .unwrap();

                return Ok(response);
            }
        } else {
            // API key is required if no session
            let response = Response::builder()
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
                .unwrap();

            return Ok(response);
        }
    }

    // Log session configuration if debug is enabled
    if config.debug.unwrap_or(false) {
        info!(
            "Session config: consciousness_mode={:?}, max_context_size={:?}",
            config.consciousness_mode, config.max_context_size
        );
    }

    let response = match method {
        Method::POST => handle_post(state, config, body, session_id).await,
        Method::GET => handle_get_sse(state, config, session_id).await,
        Method::DELETE => handle_delete_session(session_id).await,
        Method::HEAD => {
            // Return OK for HEAD requests (used by Smithery for health checks)
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(axum::body::Body::empty())
                .unwrap())
        }
        Method::OPTIONS => {
            // Handle CORS preflight with proper headers for Smithery
            Ok(Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(axum::body::Body::empty())
                .unwrap())
        }
        _ => Ok(StatusCode::METHOD_NOT_ALLOWED.into_response()),
    }?;

    Ok(response)
}

/// Handle POST requests with JSON-RPC payloads
async fn handle_post(
    state: AppState,
    mut config: SessionConfig,
    body: Option<String>,
    session_id: Option<String>,
) -> Result<Response, StatusCode> {
    let body = body.ok_or(StatusCode::BAD_REQUEST)?;

    // Parse JSON-RPC request
    let request: JsonRpcRequest = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to parse JSON-RPC request: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    debug!(
        "Received MCP request: method={}, id={:?}",
        request.method, request.id
    );

    // For non-initialize requests, validate session
    if request.method != "initialize" {
        if let Some(sid) = &session_id {
            if let Some(mut session) = SESSIONS.get_mut(sid) {
                // Update last accessed time
                session.last_accessed = std::time::Instant::now();
                // Use session's config
                config = session.config.clone();
                info!("Using existing session: {}", sid);
            } else {
                warn!("Invalid session ID: {}", sid);
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(
                        Json(json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32000,
                                "message": "Invalid session ID"
                            },
                            "id": request.id
                        }))
                        .into_response()
                        .into_body(),
                    )
                    .unwrap());
            }
        } else if request.method != "notifications/initialized" {
            // Session ID required for non-initialize, non-notification requests
            warn!("Missing session ID for method: {}", request.method);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "application/json")
                .body(
                    Json(json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32000,
                            "message": "Session ID required"
                        },
                        "id": request.id
                    }))
                    .into_response()
                    .into_body(),
                )
                .unwrap());
        }
    }

    // Store method for later use
    let method = request.method.clone();

    // Route to appropriate handler
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, request, &config).await,
        "notifications/initialized" => handle_initialized(&state, request).await,
        "tools/list" => handle_tools_list(&state, request).await,
        "tools/call" => handle_tool_call(&state, request, config.agent_role.as_deref()).await,
        "prompts/list" => handle_prompts_list(&state, request).await,
        "prompts/get" => handle_prompts_get(&state, request).await,
        "resources/list" => handle_resources_list(&state, request).await,
        "resources/read" => handle_resources_read(&state, request).await,
        "resources/subscribe" => handle_resources_subscribe(&state, request).await,
        "resources/unsubscribe" => handle_resources_unsubscribe(&state, request).await,
        "sampling/createMessage" => handle_sampling_create(&state, request).await,
        "completion/complete" => handle_completion(&state, request).await,
        "ping" => handle_ping(request).await,
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

    // Check if this is an initialize response that includes a sessionId
    let mut session_id: Option<String> = None;
    if method == "initialize" {
        if let Some(result) = &response.result {
            if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
                session_id = Some(sid.to_string());
            }
        }
    }

    // Create the response
    let mut response_builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    // Add session ID header if present
    if let Some(sid) = session_id {
        response_builder = response_builder.header("Mcp-Session-Id", sid);
    }

    // Add protocol version header
    response_builder = response_builder.header("Mcp-Protocol-Version", "2024-11-05");

    let response = response_builder
        .body(Json(response).into_response().into_body())
        .unwrap();

    Ok(response)
}

/// Handle GET requests for SSE stream
async fn handle_get_sse(
    _state: AppState,
    _config: SessionConfig,
    session_id: Option<String>,
) -> Result<Response, StatusCode> {
    // Validate session for GET requests
    if let Some(sid) = &session_id {
        if let Some(mut session) = SESSIONS.get_mut(sid) {
            // Update last accessed time
            session.last_accessed = std::time::Instant::now();
            info!("SSE stream for session: {}", sid);
        } else {
            warn!("Invalid session ID for SSE: {}", sid);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from("Invalid session ID"))
                .unwrap());
        }
    } else {
        warn!("Missing session ID for SSE stream");
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::from("Session ID required"))
            .unwrap());
    }
    // For Smithery's Streamable HTTP, we need to return a simple SSE stream
    // that will handle JSON-RPC messages sent as events
    let (_tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);

    // Don't send any initial events - let the client initiate
    // This matches the Streamable HTTP specification

    // Convert receiver to stream
    let stream = ReceiverStream::new(rx);

    // Set up SSE response with appropriate headers
    let response = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keep-alive"),
    );

    Ok(response.into_response())
}

/// Handle DELETE requests for session termination
async fn handle_delete_session(session_id: Option<String>) -> Result<Response, StatusCode> {
    if let Some(sid) = session_id {
        if let Some(_) = SESSIONS.remove(&sid) {
            info!("Session terminated: {}", sid);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(axum::body::Body::empty())
                .unwrap())
        } else {
            warn!("Attempted to delete non-existent session: {}", sid);
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(axum::body::Body::from("Session not found"))
                .unwrap())
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::from("Session ID required"))
            .unwrap())
    }
}

/// Handle initialize request
async fn handle_initialize(
    _state: &AppState,
    request: JsonRpcRequest,
    config: &SessionConfig,
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

    info!(
        "MCP initialize: protocol_version={}, client_info={:?}",
        params.protocol_version, params.client_info
    );

    // Check protocol version compatibility
    let supported_version = "2024-11-05";
    let negotiated_version = if params.protocol_version == supported_version {
        supported_version
    } else {
        // For now, we only support one version
        warn!(
            "Client requested unsupported protocol version: {}",
            params.protocol_version
        );
        supported_version
    };

    let sampling_enabled = sampling_feature_enabled();

    // Build server capabilities
    let mut server_capabilities = json!({
        "tools": {
            "listChanged": true
        },
        "prompts": {
            "listChanged": true
        },
        "resources": {
            "listChanged": true,
            "subscribe": true
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

    if sampling_enabled {
        if let Some(map) = server_capabilities.as_object_mut() {
            map.insert(
                "sampling".to_string(),
                json!({
                    "clientSide": true,
                    "notes": "Sampling requests are routed to the client's LLM"
                }),
            );
        }
    }

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

    // Generate a session ID for streamable-http transport
    let session_id = format!("mop-{}", uuid::Uuid::new_v4());

    // Store the session
    let session_data = SessionData {
        id: session_id.clone(),
        config: config.clone(),
        created_at: std::time::Instant::now(),
        last_accessed: std::time::Instant::now(),
    };
    SESSIONS.insert(session_id.clone(), session_data);
    info!("Created new session: {}", session_id);

    // Store session ID in the result for HTTP transport
    let mut response = create_success_response(request.id, result);

    // Add session ID to response headers (will be handled by the HTTP layer)
    if let serde_json::Value::Object(ref mut map) = response.result.as_mut().unwrap() {
        map.insert("sessionId".to_string(), json!(session_id));
    }

    response
}

/// Handle initialized notification
async fn handle_initialized(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("MCP client initialized");

    // This is a notification, so we don't send a response
    // But since we're in HTTP mode, we'll send an empty success
    create_success_response(request.id, json!({}))
}

/// Handle tools/list request
async fn handle_tools_list(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Listing MCP tools");

    // Get tools from registry
    let tools = state.tool_registry.get_all_tools();

    // Convert to MCP tool format
    let mcp_tools: Vec<Value> = tools
        .into_iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema,
                "outputSchema": tool.output_schema
            })
        })
        .collect();

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

    // Execute the tool based on its name
    let tool_response = match params.name.as_str() {
        "exa_search_example" => execute_exa_search_example(state, augmented_args).await,
        "exa_research_example" => execute_exa_research_example(state, augmented_args).await,
        "orchestrate_mcp_proxy" => execute_orchestrate_mcp_proxy(state, augmented_args).await,
        "discover_mcp_tools" => execute_discover_mcp_tools(state, augmented_args).await,
        _ => {
            // Check if it's a federated tool
            if let Some(federation_manager) = state.federation_manager.read().await.as_ref() {
                match federation_manager
                    .route_tool_call(
                        &params.name,
                        augmented_args.clone(),
                        crate::federation::ExecutionMode::Execute,
                    )
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        json!({
                            "error": format!("Tool execution failed: {}", e),
                            "tool": params.name,
                            "augmented_arguments": augmented_args
                        })
                    }
                }
            } else {
                json!({
                    "error": format!("Unknown tool: {}", params.name),
                    "available_tools": ["exa_search_example", "exa_research_example", "orchestrate_mcp_proxy", "discover_mcp_tools"]
                })
            }
        }
    };

    // Process the response through the shim
    let processed_response = {
        let shim = state.pitfall_shim.read().await;
        match shim.process_response(&params.name, &tool_response) {
            Ok(processed) => processed,
            Err(e) => {
                warn!("Failed to process response with shim: {}", e);
                tool_response
            }
        }
    };

    create_success_response(
        request.id,
        json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&processed_response).unwrap_or_default()
            }],
            "isError": false
        }),
    )
}

/// Handle completion request
async fn handle_completion(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
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
fn build_mcp_config() -> serde_json::Value {
    let sampling_enabled = sampling_feature_enabled();

    let mut capabilities = serde_json::Map::new();
    capabilities.insert("tools".to_string(), serde_json::Value::Bool(true));
    capabilities.insert("prompts".to_string(), serde_json::Value::Bool(true));
    capabilities.insert("resources".to_string(), serde_json::Value::Bool(true));
    if sampling_enabled {
        capabilities.insert("sampling".to_string(), serde_json::Value::Bool(true));
    }

    let mut feature_flags = serde_json::Map::new();
    if sampling_enabled {
        feature_flags.insert("samplingEnabled".to_string(), serde_json::Value::Bool(true));
        feature_flags.insert(
            "samplingRequiresClientLLM".to_string(),
            serde_json::Value::Bool(true),
        );
    }

    json!({
        "name": "meta-orchestration-protocol",
        "title": "Meta-Orchestration Protocol (MOP) Server",
        "description": "Consciousness-aware MCP orchestration framework",
        "version": env!("CARGO_PKG_VERSION"),
        "vendor": "Prompted LLC",
        "homepage": "https://github.com/prompted365/context-casial-xpress",
        "transport": ["streamable-http"],
        "capabilities": serde_json::Value::Object(capabilities),
        "featureFlags": serde_json::Value::Object(feature_flags),
        "configSchema": {
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": "https://swarm.mop.quest/.well-known/mcp-config",
            "title": "MCP Session Configuration",
            "description": "Configuration for connecting to Meta-Orchestration Protocol server",
            "x-query-style": "dot+bracket",
            "type": "object",
            "required": ["apiKey"],
            "additionalProperties": false,
            "properties": {
                "apiKey": {
                    "type": "string",
                    "title": "API Key",
                    "description": "Your API key for authentication (DEMO KEY – public). Override by setting MOP_API_KEY. Required unless you supply an Authorization: Bearer header.",
                    "default": format!("${{MOP_API_KEY:-{}}}", DEMO_API_KEY)
                },
                "agent_role": {
                    "type": "string",
                    "title": "Agent Role",
                    "description": "Role of the calling agent",
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
                    "description": "Enable automatic context injection",
                    "default": true
                },
                "debug": {
                    "type": "boolean",
                    "title": "Debug Mode",
                    "description": "Enable debug logging",
                    "default": false
                }
            }
        }
    })
}

pub async fn well_known_config_handler(
    method: Method,
    State(state): State<AppState>,
    headers: http::HeaderMap,
    body: Option<String>,
) -> Result<Response, StatusCode> {
    match method {
        Method::GET => {
            let config = build_mcp_config();
            Ok(Json(config).into_response())
        }
        Method::POST => {
            // For POST requests, handle as JSON-RPC (Smithery might be sending JSON-RPC to this endpoint)
            if let Some(body) = body {
                // Try to parse as JSON-RPC
                if let Ok(_request) = serde_json::from_str::<JsonRpcRequest>(&body) {
                    // Forward to the regular MCP handler
                    return mcp_handler(
                        Method::POST,
                        State(state),
                        headers,
                        Query(QueryParams::default()),
                        Some(body),
                    )
                    .await;
                }
            }

            // If not JSON-RPC, return the same config as GET
            let config = build_mcp_config();
            Ok(Json(config).into_response())
        }
        _ => Ok(StatusCode::METHOD_NOT_ALLOWED.into_response()),
    }
}

// Tool execution implementations

async fn execute_exa_search_example(
    _state: &AppState,
    args: serde_json::Value,
) -> serde_json::Value {
    // Extract parameters
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let num_results = args.get("numResults").and_then(|v| v.as_u64()).unwrap_or(5);

    // Since this is an example tool, we'll return a simulated response
    // In a real implementation, this would call the actual Exa API
    json!({
        "status": "success",
        "tool": "exa_search_example",
        "query": query,
        "results": [
            {
                "title": "AI Orchestration Best Practices 2025",
                "url": "https://example.com/ai-orchestration",
                "snippet": "Latest developments in AI orchestration for microservices...",
                "score": 0.95
            },
            {
                "title": "MCP Federation Architecture Guide",
                "url": "https://example.com/mcp-federation",
                "snippet": "How to build federated MCP systems with consciousness-aware features...",
                "score": 0.92
            }
        ],
        "metadata": {
            "num_results_requested": num_results,
            "augmented": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

async fn execute_exa_research_example(
    _state: &AppState,
    args: serde_json::Value,
) -> serde_json::Value {
    let instructions = args
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let model = args
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("exa-research");

    // Simulated research task response
    json!({
        "status": "success",
        "tool": "exa_research_example",
        "task_id": uuid::Uuid::new_v4().to_string(),
        "instructions": instructions,
        "model": model,
        "result": {
            "summary": "Research task initiated. In a real implementation, this would start an async research process.",
            "next_step": "Poll for results using the task_id"
        },
        "metadata": {
            "augmented": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

async fn execute_orchestrate_mcp_proxy(
    _state: &AppState,
    args: serde_json::Value,
) -> serde_json::Value {
    let target_server = args
        .get("target_server")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tool_name = args.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
    let original_params = args.get("original_params").cloned().unwrap_or(json!({}));
    let augmentation_config = args
        .get("augmentation_config")
        .cloned()
        .unwrap_or(json!({}));

    // Apply augmentation based on config
    let mut augmented_params = original_params.clone();

    if augmentation_config
        .get("inject_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        augmented_params["_context"] = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "orchestration_source": "mop",
            "consciousness_aware": true
        });
    }

    if let Some(instructions) = augmentation_config
        .get("add_swarm_instructions")
        .and_then(|v| v.as_array())
    {
        augmented_params["_swarm_instructions"] = serde_json::Value::Array(instructions.clone());
    }

    // In a real implementation, this would forward to the actual target server
    // For now, return a response showing what would be sent
    json!({
        "status": "success",
        "tool": "orchestrate_mcp_proxy",
        "forwarded_to": target_server,
        "tool_called": tool_name,
        "augmented_params": augmented_params,
        "augmentation_applied": augmentation_config,
        "result": {
            "message": "In production, this would forward the augmented request to the target MCP server",
            "would_call": format!("{}/{}", target_server, tool_name)
        },
        "metadata": {
            "augmented": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

async fn execute_discover_mcp_tools(
    state: &AppState,
    args: serde_json::Value,
) -> serde_json::Value {
    let server_url = args
        .get("server_url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let analyze_for_orchestration = args
        .get("analyze_for_orchestration")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Get tools from our registry as an example
    let tools = state.tool_registry.get_all_tools();

    let discovered_tools: Vec<serde_json::Value> = tools
        .into_iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "source": match &tool.source {
                    crate::registry::ToolSource::Local => "local",
                    crate::registry::ToolSource::Federated { server_id, .. } => server_id
                },
                "input_schema": tool.input_schema,
                "orchestration_hints": if analyze_for_orchestration {
                    Some(json!({
                        "supports_consciousness": true,
                        "paradox_tolerant": true,
                        "federation_ready": true
                    }))
                } else {
                    None
                }
            })
        })
        .collect();

    json!({
        "status": "success",
        "tool": "discover_mcp_tools",
        "server_url": server_url,
        "discovered_tools": discovered_tools,
        "total_tools": discovered_tools.len(),
        "analysis": {
            "orchestration_compatible": true,
            "consciousness_features": ["temporal_awareness", "context_injection", "paradox_handling"],
            "recommended_patterns": ["saga", "event_driven", "federation"]
        },
        "metadata": {
            "augmented": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

// Prompts handlers

async fn handle_prompts_list(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Listing MCP prompts");

    let prompts = vec![
        json!({
            "name": "orchestrate_workflow",
            "title": "Orchestrate Multi-Agent Workflow",
            "description": "Design and execute a multi-agent workflow using MOP's consciousness-aware orchestration",
            "arguments": [
                {
                    "name": "goal",
                    "description": "The goal to achieve through orchestration",
                    "required": true
                },
                {
                    "name": "agents",
                    "description": "List of agent types needed (planner, executor, reviewer, etc)",
                    "required": false
                }
            ]
        }),
        json!({
            "name": "analyze_mcp_server",
            "title": "Analyze MCP Server Capabilities",
            "description": "Analyze an MCP server's tools, resources, and capabilities to design optimal orchestration",
            "arguments": [
                {
                    "name": "server_url",
                    "description": "URL of the MCP server to analyze",
                    "required": true
                }
            ]
        }),
        json!({
            "name": "consciousness_reflection",
            "title": "Consciousness-Aware Reflection",
            "description": "Reflect on current context and paradoxes to enhance orchestration awareness",
            "arguments": []
        }),
    ];

    create_success_response(request.id, json!({ "prompts": prompts }))
}

async fn handle_prompts_get(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct PromptsGetParams {
        name: String,
        arguments: Option<serde_json::Value>,
    }

    let params: PromptsGetParams = match serde_json::from_value(request.params) {
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

    let messages = match params.name.as_str() {
        "orchestrate_workflow" => {
            let goal = params
                .arguments
                .as_ref()
                .and_then(|a| a.get("goal"))
                .and_then(|g| g.as_str())
                .unwrap_or("achieve complex task");

            vec![json!({
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "I need to orchestrate a multi-agent workflow to: {}

\
                            Please design an orchestration plan using MOP's consciousness-aware features:
\
                            1. Identify required agents and their roles
\
                            2. Define the workflow sequence
\
                            3. Specify which MCP tools each agent needs
\
                            4. Include reflection and paradox handling steps
\
                        5. Design feedback loops for adaptation",
                        goal
                    )
                }
            })]
        }
        "analyze_mcp_server" => {
            let server_url = params
                .arguments
                .as_ref()
                .and_then(|a| a.get("server_url"))
                .and_then(|u| u.as_str())
                .unwrap_or("unknown");

            vec![json!({
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "Analyze the MCP server at {} and provide:

\
                            1. Available tools and their orchestration potential
\
                            2. Resources that can be leveraged for context
\
                            3. Prompts that enable higher-level workflows
\
                            4. How sampling can create recursive intelligence
\
                            5. Optimal orchestration patterns for this server
\
                        6. Integration strategies with other MCP servers",
                        server_url
                    )
                }
            })]
        }
        "consciousness_reflection" => {
            vec![json!({
                "role": "user",
                "content": {
                    "type": "text",
                    "text": "Engage in consciousness-aware reflection:

\
                            1. What paradoxes exist in the current orchestration context?
\
                            2. How can we leverage these paradoxes to strengthen the system?
\
                            3. What emergent behaviors are appearing in the multi-agent coordination?
\
                            4. How can we enhance self-awareness in the orchestration loop?
\
                        5. What meta-patterns can guide future orchestrations?"
                }
            })]
        }
        _ => {
            return create_error_response(
                request.id,
                -32602,
                &format!("Unknown prompt: {}", params.name),
                None,
            );
        }
    };

    create_success_response(request.id, json!({ "messages": messages }))
}

// Resources handlers

async fn handle_resources_list(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Listing MCP resources");

    let resources = vec![
        json!({
            "uri": "mop://orchestration/context",
            "name": "Current Orchestration Context",
            "title": "Live Orchestration Context",
            "description": "Real-time context including active agents, workflows, and paradoxes",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "mop://orchestration/history",
            "name": "Orchestration History",
            "title": "Historical Orchestration Data",
            "description": "Past orchestrations, patterns, and learnings",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "mop://consciousness/state",
            "name": "Consciousness State",
            "title": "Current Consciousness Metrics",
            "description": "Paradox levels, awareness metrics, and substrate operations",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "mop://federation/servers",
            "name": "Federated Servers",
            "title": "Connected MCP Servers",
            "description": "List of federated MCP servers and their capabilities",
            "mimeType": "application/json"
        }),
    ];

    create_success_response(request.id, json!({ "resources": resources }))
}

async fn handle_resources_read(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct ResourcesReadParams {
        uri: String,
    }

    let params: ResourcesReadParams = match serde_json::from_value(request.params) {
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

    let contents = match params.uri.as_str() {
        "mop://orchestration/context" => {
            let metrics = state.metrics_collector.read().await.get_current_metrics();
            vec![json!({
                "uri": params.uri.clone(),
                "mimeType": "text/plain",
                "text": serde_json::to_string_pretty(&json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "active_sessions": state.active_sessions.len(),
                    "coordination_events": metrics.coordination_events,
                    "paradoxes_resolved": metrics.paradoxes_resolved,
                    "current_awareness": {
                        "temporal": true,
                        "contextual": true,
                        "paradox_tolerant": true
                    },
                    "orchestration_mode": "consciousness-aware",
                    "shim_active": state.pitfall_shim.read().await.is_enabled()
                })).unwrap()
            })]
        }
        "mop://consciousness/state" => {
            let metrics = state.metrics_collector.read().await.get_current_metrics();
            vec![json!({
                "uri": params.uri.clone(),
                "mimeType": "text/plain",
                "text": serde_json::to_string_pretty(&json!({
                    "consciousness_metrics": {
                        "paradox_resolution_rate": metrics.paradoxes_resolved,
                        "perception_locks": metrics.perception_locks,
                        "substrate_operations": metrics.substrate_operations,
                        "awareness_level": "high"
                    },
                    "emergence_patterns": [
                        "recursive_self_improvement",
                        "paradox_strengthening",
                        "context_amplification"
                    ]
                })).unwrap()
            })]
        }
        "mop://federation/servers" => {
            let federation_info = if let Some(fed) = state.federation_manager.read().await.as_ref()
            {
                json!({
                    "federated_servers": fed.get_active_servers().await,
                    "total_tools": state.tool_registry.get_all_tools().len()
                })
            } else {
                json!({
                    "federated_servers": [],
                    "federation_enabled": false
                })
            };

            vec![json!({
                "uri": params.uri.clone(),
                "mimeType": "text/plain",
                "text": serde_json::to_string_pretty(&federation_info).unwrap()
            })]
        }
        _ => {
            return create_error_response(
                request.id,
                -32602,
                &format!("Unknown resource: {}", params.uri),
                None,
            );
        }
    };

    create_success_response(request.id, json!({ "contents": contents }))
}

async fn handle_resources_subscribe(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    // For now, acknowledge subscription but don't implement real-time updates
    info!("Resource subscription requested");
    create_success_response(request.id, json!({}))
}

async fn handle_resources_unsubscribe(
    _state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    info!("Resource unsubscription requested");
    create_success_response(request.id, json!({}))
}

// Sampling handler

async fn handle_sampling_create(_state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct SamplingCreateParams {
        messages: Vec<serde_json::Value>,
        #[serde(rename = "systemPrompt")]
        system_prompt: Option<String>,
        #[serde(rename = "modelPreferences")]
        model_preferences: Option<serde_json::Value>,
    }

    let params: SamplingCreateParams = match serde_json::from_value(request.params) {
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

    if !sampling_feature_enabled() {
        return create_error_response(
            request.id,
            -32001,
            "Sampling disabled by server configuration",
            Some(json!({
                "featureFlag": "MOP_ENABLE_SAMPLING",
                "enabled": false,
                "action": "Set MOP_ENABLE_SAMPLING=1 to allow clients to expose sampling tools"
            })),
        );
    }

    // This is where MOP would delegate back to the client's LLM
    // For now, return an error indicating this needs client-side implementation
    create_error_response(
        request.id,
        -32601,
        "Sampling requires client-side LLM integration",
        Some(json!({
            "note": "MOP server requested sampling, but this requires the client to provide LLM access",
            "messages": params.messages,
            "system_prompt": params.system_prompt,
            "samplingEnabled": true
        })),
    )
}

async fn handle_ping(request: JsonRpcRequest) -> JsonRpcResponse {
    // Simple ping response - just acknowledge
    create_success_response(request.id, json!({}))
}
