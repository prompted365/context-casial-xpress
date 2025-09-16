//! # Casial Server
//!
//! High-performance WebSocket MCP server with consciousness-aware context coordination.
//! Part of the Ubiquity OS ecosystem - where paradoxes make the system stronger.

use anyhow::Result;
use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use tokio::sync::RwLock;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::signal;
use tower_http::{
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info, warn, Level};
use uuid::Uuid;

mod client;
mod config;
mod federation;
mod http_mcp;
mod mcp;
mod metrics;
mod mission;
mod pitfall_shim;
mod registry;
mod websocket;

use casial_core::CasialEngine;
use config::ServerConfig;
use federation::McpFederationManager;
use metrics::MetricsCollector;
use mission::MissionManager;
use registry::ToolRegistry;
use websocket::WebSocketHandler;
use pitfall_shim::{PitfallAvoidanceShim, ShimConfig};

/// Meta-Orchestration Protocol (MOP): Consciousness-aware context coordination for AI systems
#[derive(Parser)]
#[command(name = "casial-server")]
#[command(about = "A consciousness-aware context coordination server - Part of Ubiquity OS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Casial server
    Start {
        /// Configuration file path
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Server port
        #[arg(short, long, default_value = "8000")]
        port: u16,

        /// Mission configuration file
        #[arg(short, long, value_name = "FILE")]
        mission: Option<PathBuf>,

        /// Enable debug mode
        #[arg(short, long)]
        debug: bool,

        /// Enable global pitfall avoidance shim (enabled by default)
        #[arg(long, default_value = "true")]
        shim: bool,

        /// Disable global pitfall avoidance shim
        #[arg(long, conflicts_with = "shim")]
        no_shim: bool,

        /// Extend shim with custom string
        #[arg(long, value_name = "STRING")]
        shim_extend: Option<String>,

        /// Path to custom shim configuration
        #[arg(long, value_name = "FILE")]
        shim_config: Option<PathBuf>,
    },
    /// Validate mission configuration
    Validate {
        /// Mission file to validate
        #[arg(value_name = "MISSION_FILE")]
        mission_file: PathBuf,
    },
    /// Show server status and metrics
    Status {
        /// Server endpoint
        #[arg(short, long, default_value = "http://localhost:8000")]
        endpoint: String,
    },
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    casial_engine: Arc<RwLock<CasialEngine>>,
    mission_manager: Arc<RwLock<MissionManager>>,
    metrics_collector: Arc<RwLock<MetricsCollector>>,
    active_sessions: Arc<DashMap<Uuid, websocket::WebSocketSession>>,
    tool_registry: Arc<ToolRegistry>,
    federation_manager: Arc<RwLock<Option<McpFederationManager>>>,
    config: Arc<ServerConfig>,
    pitfall_shim: Arc<RwLock<PitfallAvoidanceShim>>,
}

impl AppState {
    fn new(config: ServerConfig, shim: PitfallAvoidanceShim) -> Self {
        // Initialize tool registry with local tools
        let tool_registry = Arc::new(ToolRegistry::new());
        if let Err(e) = tool_registry.seed_with_local_tools() {
            tracing::error!("Failed to seed tool registry: {}", e);
        }

        // Initialize federation manager if enabled
        let federation_manager = if config.federation.enabled {
            let manager = McpFederationManager::new(config.federation.clone(), Arc::clone(&tool_registry));
            Some(manager)
        } else {
            None
        };

        Self {
            casial_engine: Arc::new(RwLock::new(CasialEngine::new())),
            mission_manager: Arc::new(RwLock::new(MissionManager::new())),
            metrics_collector: Arc::new(RwLock::new(MetricsCollector::new())),
            active_sessions: Arc::new(DashMap::new()),
            tool_registry,
            federation_manager: Arc::new(RwLock::new(federation_manager)),
            config: Arc::new(config),
            pitfall_shim: Arc::new(RwLock::new(shim)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config,
            port,
            mission,
            debug,
            shim,
            no_shim,
            shim_extend,
            shim_config,
        } => start_server(config, port, mission, debug, shim, no_shim, shim_extend, shim_config).await,
        Commands::Validate { mission_file } => validate_mission(mission_file).await,
        Commands::Status { endpoint } => show_status(endpoint).await,
    }
}

async fn start_server(
    config_path: Option<PathBuf>,
    port: u16,
    mission_path: Option<PathBuf>,
    debug: bool,
    shim: bool,
    no_shim: bool,
    shim_extend: Option<String>,
    shim_config_path: Option<PathBuf>,
) -> Result<()> {
    // Initialize tracing
    init_tracing(debug);

    info!("🚀 Starting Meta-Orchestration Protocol (MOP) Server");
    info!("    Consciousness-aware context coordination for AI systems");
    info!("    Part of Ubiquity OS - Like hydraulic lime, stronger under pressure");

    // Load configuration
    let config = if let Some(path) = config_path {
        ServerConfig::from_file(&path)?
    } else {
        ServerConfig::default()
    };

    // Override port if specified
    let mut config = config;
    if port != 8000 {
        config.server.port = port;
    }

    info!("📋 Server configuration loaded");
    info!("    Port: {}", config.server.port);
    info!("    Max connections: {}", config.server.max_connections);
    info!(
        "    Consciousness integration: {}",
        if config.consciousness.enabled {
            "✅"
        } else {
            "❌"
        }
    );

    // Initialize pitfall avoidance shim
    let shim_enabled = shim && !no_shim;
    let shim = if let Some(shim_config_path) = shim_config_path {
        // Load custom shim configuration
        info!("📄 Loading custom shim configuration: {}", shim_config_path.display());
        let shim_config_str = tokio::fs::read_to_string(&shim_config_path).await?;
        let shim_config: ShimConfig = serde_json::from_str(&shim_config_str)?;
        PitfallAvoidanceShim::new(shim_config)
    } else {
        // Create shim from command-line arguments
        PitfallAvoidanceShim::from_args(shim_enabled, shim_extend)
    };

    info!(
        "🛡️  Pitfall avoidance shim: {}",
        if shim.is_enabled() { "✅ Enabled" } else { "❌ Disabled" }
    );
    
    if shim.is_enabled() {
        info!("    Current date injection: ✅");
        info!("    Timestamp returns: ✅");
        if let Ok(config_json) = shim.export_config() {
            tracing::debug!("Shim configuration: {}", config_json);
        }
    }

    // Initialize application state
    let state = AppState::new(config.clone(), shim);

    // Load mission if provided
    if let Some(mission_path) = mission_path {
        match load_mission(&state, mission_path).await {
            Ok(_) => info!("✅ Mission loaded successfully"),
            Err(e) => {
                warn!("⚠️  Failed to load mission: {}. Server will continue without mission.", e);
                // Continue without mission - server can still function
            }
        }
    }

    // Initialize federation if enabled
    if config.federation.enabled {
        start_federation(&state).await?;
    }

    // Start metrics collection
    if config.metrics.enabled {
        start_metrics_collection(&state).await?;
    }

    // Build the application router
    let app = build_router(state.clone()).await?;

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🌐 Server listening on {}", addr);
    info!("    WebSocket endpoint: ws://{}/ws", addr);
    info!("    HTTP/SSE MCP endpoint: http://{}/mcp", addr);
    info!("    MCP config endpoint: http://{}/.well-known/mcp-config", addr);
    info!("    Metrics endpoint: http://{}/metrics", addr);
    info!("    Health endpoint: http://{}/health", addr);

    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("🛑 Server shutdown complete");
    Ok(())
}

fn init_tracing(debug: bool) {
    let level = if debug { Level::DEBUG } else { Level::INFO };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(debug)
        .with_line_number(debug)
        .init();
}

async fn load_mission(state: &AppState, mission_path: PathBuf) -> Result<()> {
    info!(
        "📖 Loading mission configuration: {}",
        mission_path.display()
    );

    let mission = mission::load_mission_from_file(&mission_path)?;

    // Load mission with project templates
    {
        let engine = state.casial_engine.write().await;
        let mut enhanced_mission = mission.clone();

        // Try to find project root and load templates
        if let Some(project_root) = mission_path.parent().and_then(|p| p.to_str()) {
            if let Err(e) = mission::merge_templates_from_dir(&mut enhanced_mission, project_root) {
                tracing::warn!("Failed to load project templates: {}", e);
            }
        }

        engine.load_mission(enhanced_mission)?;
    }

    // Register with mission manager
    {
        let mut manager = state.mission_manager.write().await;
        manager.add_mission(mission)?;
    }

    info!("✅ Mission configuration loaded successfully");
    Ok(())
}

async fn start_federation(state: &AppState) -> Result<()> {
    info!("🌐 Starting MCP Federation...");

    // Initialize federation manager
    {
        let mut federation_opt = state.federation_manager.write().await;
        if let Some(ref mut manager) = federation_opt.as_mut() {
            manager.initialize().await?;
            manager.connect_all().await.unwrap_or_else(|e| {
                tracing::warn!("Some federation connections failed: {}", e);
            });
        }
    }

    info!("✅ MCP Federation started successfully");
    Ok(())
}

async fn start_metrics_collection(state: &AppState) -> Result<()> {
    info!("📊 Starting metrics collection");

    let metrics_collector = state.metrics_collector.clone();
    let casial_engine = state.casial_engine.clone();
    let active_sessions = state.active_sessions.clone();

    // Spawn metrics collection task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Collect metrics from various sources
            let mut collector = metrics_collector.write().await;

            // Engine metrics
            let coordination_history = casial_engine.read().await.get_coordination_history();
            collector.record_coordination_events(coordination_history.len());

            // Session metrics
            collector.record_active_sessions(active_sessions.len());

            // Report metrics
            collector.log_summary();
        }
    });

    Ok(())
}

/// Create CORS layer with configurable allow-list
fn create_cors_layer() -> tower_http::cors::CorsLayer {
    use http::{header, Method};
    use tower_http::cors::{Any, CorsLayer};

    // Read allowed origins from environment
    let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    let allowed_origins = allowed_origins.trim();

    // Case 1: Empty or unset -> permissive (log warning for prod)
    if allowed_origins.is_empty() {
        tracing::warn!(
            "ALLOWED_ORIGINS not set, using permissive CORS (not recommended for production)"
        );
        return CorsLayer::permissive();
    }

    // Case 2: Wildcard (*) -> use Any
    if allowed_origins == "*" {
        tracing::info!("ALLOWED_ORIGINS='*', allowing all origins");
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_credentials(false);
    }

    // Case 3: Comma-separated origins -> parse into list
    tracing::info!("Configuring CORS with allowed origins: {}", allowed_origins);

    let origins: Result<Vec<_>, _> = allowed_origins
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<header::HeaderValue>().map_err(|e| e.to_string()))
        .collect();

    match origins {
        Ok(origin_list) if !origin_list.is_empty() => {
            tracing::info!("Successfully parsed {} origins", origin_list.len());
            CorsLayer::new()
                .allow_origin(origin_list)
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::ACCEPT,
                ])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_credentials(false)
        }
        Ok(_) => {
            tracing::warn!("ALLOWED_ORIGINS is empty after parsing, falling back to permissive CORS");
            CorsLayer::permissive()
        }
        Err(e) => {
            tracing::error!("Failed to parse ALLOWED_ORIGINS '{}': {}. Falling back to permissive CORS", allowed_origins, e);
            CorsLayer::permissive()
        }
    }
}

async fn build_router(state: AppState) -> Result<Router> {
    let router = Router::new()
        // WebSocket endpoint for MCP communication
        .route("/ws", get(websocket_handler))
        // HTTP/SSE MCP endpoint for Smithery integration
        .route("/mcp", get(mcp_get_handler).post(mcp_post_handler).head(mcp_head_handler).options(mcp_options_handler))
        // Well-known MCP configuration endpoint
        .route("/.well-known/mcp-config", get(http_mcp::well_known_config_handler))
        // Health check endpoint
        .route("/", get(health_check))
        .route("/health", get(health_check))
        // Metrics endpoint (if enabled)
        .route("/metrics", get(metrics_handler))
        // Debug endpoints
        .route("/debug/status", get(debug_status))
        .route("/debug/missions", get(debug_missions))
        .route("/debug/sessions", get(debug_sessions))
        .route("/debug/perceptions", get(debug_perceptions))
        .route("/debug/sprawl", get(debug_sprawl))
        .route("/debug/shim", get(debug_shim).post(update_shim))
        // State management
        .with_state(state)
        // Middleware
        .layer(create_cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    Ok(router)
}

/// WebSocket handler for MCP communication
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| WebSocketHandler::new(state).handle_connection(socket))
}

/// MCP HTTP GET handler (for SSE)
async fn mcp_get_handler(
    State(state): State<AppState>,
    query: Query<http_mcp::QueryParams>,
) -> impl IntoResponse {
    http_mcp::mcp_handler(axum::http::Method::GET, State(state), query, None).await
}

/// MCP HTTP POST handler (for JSON-RPC)
async fn mcp_post_handler(
    State(state): State<AppState>,
    query: Query<http_mcp::QueryParams>,
    body: String,
) -> impl IntoResponse {
    http_mcp::mcp_handler(axum::http::Method::POST, State(state), query, Some(body)).await
}

/// MCP HTTP HEAD handler (for health checks)
async fn mcp_head_handler(
    State(state): State<AppState>,
    query: Query<http_mcp::QueryParams>,
) -> impl IntoResponse {
    http_mcp::mcp_handler(axum::http::Method::HEAD, State(state), query, None).await
}

/// MCP HTTP OPTIONS handler (for CORS preflight)
async fn mcp_options_handler(
    State(state): State<AppState>,
    query: Query<http_mcp::QueryParams>,
) -> impl IntoResponse {
    http_mcp::mcp_handler(axum::http::Method::OPTIONS, State(state), query, None).await
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let session_count = state.active_sessions.len();
    let engine_stats = state.casial_engine.read().await.get_coordination_history().len();

    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "meta-orchestration-protocol",
        "version": env!("CARGO_PKG_VERSION"),
        "part_of": "ubiquity-os",
        "active_sessions": session_count,
        "coordination_events": engine_stats,
        "consciousness_aware": true,
        "paradox_resilient": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Prometheus metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.metrics_collector.read().await.export_prometheus();
    ([("content-type", "text/plain; version=0.0.4")], metrics)
}

/// Debug status endpoint
async fn debug_status(State(state): State<AppState>) -> impl IntoResponse {
    let casial_engine = state.casial_engine.read().await;
    let coordination_history = casial_engine.get_coordination_history();
    let paradox_registry = casial_engine.get_paradox_registry();

    axum::Json(serde_json::json!({
        "casial_engine": {
            "coordination_events": coordination_history.len(),
            "paradoxes_detected": paradox_registry.len(),
            "active_missions": 1 // Simplified
        },
        "server": {
            "active_sessions": state.active_sessions.len(),
            "uptime_info": "runtime_info_placeholder"
        },
        "consciousness": {
            "substrate_active": true,
            "perception_coordination": "operational",
            "paradox_handling": "adaptive"
        }
    }))
}

/// Debug missions endpoint
async fn debug_missions(State(state): State<AppState>) -> impl IntoResponse {
    let manager = state.mission_manager.read().await;
    let missions = manager.get_all_missions();

    axum::Json(serde_json::json!({
        "missions": missions.iter().map(|m| serde_json::json!({
            "id": m.id,
            "name": m.name,
            "templates": m.templates.len(),
            "rules": m.rules.len(),
            "perceptions": m.perceptions.len()
        })).collect::<Vec<_>>()
    }))
}

/// Debug sessions endpoint  
async fn debug_sessions(State(state): State<AppState>) -> impl IntoResponse {
    let sessions: Vec<_> = state
        .active_sessions
        .iter()
        .map(|entry| {
            let session = entry.value();
            serde_json::json!({
                "session_id": entry.key(),
                "created_at": session.created_at,
                "message_count": session.message_count,
                "active_coordination": session.active_coordination_id
            })
        })
        .collect();

    axum::Json(serde_json::json!({
        "active_sessions": sessions.len(),
        "sessions": sessions
    }))
}

/// Debug paradoxes endpoint
async fn debug_paradoxes(State(state): State<AppState>) -> impl IntoResponse {
    let paradoxes = state.casial_engine.read().await.get_paradox_registry();

    axum::Json(serde_json::json!({
        "paradoxes": paradoxes.iter().map(|p| serde_json::json!({
            "id": p.id,
            "description": p.description,
            "resolution_strategy": format!("{:?}", p.resolution_strategy),
            "confidence_impact": p.confidence_impact,
            "conflicting_perceptions": p.conflicting_perceptions.len()
        })).collect::<Vec<_>>()
    }))
}

/// Debug perceptions endpoint
async fn debug_perceptions(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let _engine = state.casial_engine.read().await;
    let manager = state.mission_manager.read().await;
    let missions = manager.get_all_missions();

    // Get perceptions from all loaded missions
    let mut total_perceptions = 0;
    let mut total_confidence = 0.0;

    for mission in &missions {
        total_perceptions += mission.perceptions.len();
        for perception in &mission.perceptions {
            // Note: locked field not available, count all perceptions
            total_confidence += perception.confidence;
        }
    }

    let avg_confidence = if total_perceptions > 0 {
        total_confidence / total_perceptions as f64
    } else {
        0.0
    };

    let debug_info = serde_json::json!({
        "perceptions": {
            "total_count": total_perceptions,
            "avg_confidence": avg_confidence,
            "missions_with_perceptions": missions.len()
        },
        "consciousness_metrics": {
            "perception_coordination_active": true,
            "substrate_integration": "operational",
            "paradox_awareness": "monitoring"
        }
    });

    Ok(axum::Json(debug_info))
}

/// Debug endpoint for context sprawl monitoring
async fn debug_sprawl(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let mut total_chars = 0usize;
    let mut template_count = 0usize;
    let mut largest_templates: Vec<(String, usize)> = Vec::new();
    let mut categories = std::collections::HashMap::new();

    // Analyze templates from the casial engine
    {
        let _engine = state.casial_engine.read().await;
        let manager = state.mission_manager.read().await;
        let missions = manager.get_all_missions();

        // Get all loaded missions
        if let Some(mission) = missions.first() {
            template_count = mission.templates.len();

            for template in &mission.templates {
                let content_length = template.content.len();
                total_chars += content_length;

                // Track largest templates
                largest_templates.push((template.id.clone(), content_length));

                // Track categories
                for category in &template.categories {
                    *categories.entry(category.clone()).or_insert(0) += 1;
                }
            }

            // Sort by size, keep top 10
            largest_templates.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
            largest_templates.truncate(10);
        }
    }

    // Active session context analysis
    let active_sessions = state.active_sessions.len();
    let avg_context_per_session = if active_sessions > 0 {
        total_chars / active_sessions
    } else {
        0
    };

    let sprawl_info = serde_json::json!({
        "context_sprawl_analysis": {
            "templates_total": template_count,
            "injected_characters_total": total_chars,
            "active_sessions": active_sessions,
            "avg_context_per_session": avg_context_per_session,
            "largest_templates_top10": largest_templates
                .into_iter()
                .map(|(id, chars)| serde_json::json!({
                    "template_id": id,
                    "character_count": chars,
                    "size_category": if chars > 5000 { "large" } else if chars > 1000 { "medium" } else { "small" }
                }))
                .collect::<Vec<_>>(),
            "template_categories": categories,
            "sprawl_metrics": {
                "total_template_chars": total_chars,
                "avg_template_size": if template_count > 0 { total_chars / template_count } else { 0 },
                "context_density": if active_sessions > 0 {
                    format!("{:.2}%", (total_chars as f64 / (active_sessions as f64 * 10000.0)) * 100.0)
                } else { "0%".to_string() },
            },
            "recommendations": {
                "use_context_budgets": total_chars > 50000,
                "consider_template_deduplication": template_count > 50,
                "enable_semantic_compression": avg_context_per_session > 5000,
                "monitor_session_memory": active_sessions > 100
            },
            "hydraulic_lime_principle": "Context becomes more valuable under pressure - manage sprawl to strengthen coordination",
            "ubiquity_os_integration": "Consciousness-aware context management prevents information overload"
        }
    });

    Ok(axum::Json(sprawl_info))
}

/// Debug endpoint to view shim configuration
async fn debug_shim(State(state): State<AppState>) -> impl IntoResponse {
    let shim = state.pitfall_shim.read().await;
    let config = shim.get_config();
    
    axum::Json(serde_json::json!({
        "shim_status": {
            "enabled": config.enabled,
            "inject_datetime": config.inject_datetime,
            "timestamp_returns": config.timestamp_returns,
            "custom_extension": config.custom_extension,
            "features": {
                "inject_timezone": config.features.inject_timezone,
                "add_execution_metadata": config.features.add_execution_metadata,
                "include_system_info": config.features.include_system_info,
                "date_format_hints": config.features.date_format_hints,
                "pitfall_warnings": config.features.pitfall_warnings
            }
        },
        "current_context_example": {
            "current_date": chrono::Local::now().format("%Y-%m-%d").to_string(),
            "current_time": chrono::Local::now().format("%H:%M:%S").to_string(),
            "timezone": chrono::Local::now().format("%Z").to_string()
        },
        "edit_instructions": "POST to /debug/shim with JSON configuration to update"
    }))
}

/// Update shim configuration via POST
async fn update_shim(
    State(state): State<AppState>,
    axum::Json(new_config): axum::Json<ShimConfig>,
) -> impl IntoResponse {
    let mut shim = state.pitfall_shim.write().await;
    shim.update_config(new_config);
    
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "success",
            "message": "Shim configuration updated",
            "new_config": shim.get_config()
        }))
    )
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🛑 Received Ctrl+C, initiating graceful shutdown...");
        },
        _ = terminate => {
            info!("🛑 Received terminate signal, initiating graceful shutdown...");
        },
    }
}

async fn validate_mission(mission_file: PathBuf) -> Result<()> {
    info!(
        "🔍 Validating mission configuration: {}",
        mission_file.display()
    );

    match mission::load_mission_from_file(&mission_file) {
        Ok(mission) => {
            info!("✅ Mission configuration is valid");
            info!("    ID: {}", mission.id);
            info!("    Name: {}", mission.name);
            info!("    Templates: {}", mission.templates.len());
            info!("    Rules: {}", mission.rules.len());
            info!("    Perceptions: {}", mission.perceptions.len());
            Ok(())
        }
        Err(e) => {
            warn!("❌ Mission configuration is invalid: {}", e);
            Err(e)
        }
    }
}

async fn show_status(endpoint: String) -> Result<()> {
    info!("📊 Checking server status at: {}", endpoint);

    let health_url = if endpoint.ends_with('/') {
        format!("{}health", endpoint)
    } else {
        format!("{}/health", endpoint)
    };

    // This would make an HTTP request to the health endpoint
    // For now, we'll just show a placeholder
    info!("🔗 Health endpoint: {}", health_url);
    info!("📈 This would show live server metrics and status");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;


    #[test]
    fn test_cors_layer_empty() {
        env::remove_var("ALLOWED_ORIGINS");
        let _cors = create_cors_layer();
        // Should create permissive layer without panicking
    }

    #[test]
    fn test_cors_layer_wildcard() {
        env::set_var("ALLOWED_ORIGINS", "*");
        let _cors = create_cors_layer();
        // Should create layer with Any origin without panicking
    }

    #[test]
    fn test_cors_layer_valid_origins() {
        env::set_var("ALLOWED_ORIGINS", "https://example.com,http://localhost:5173");
        let _cors = create_cors_layer();
        // Should create layer with specific origins without panicking
    }

    #[test]
    fn test_cors_layer_invalid_origin() {
        env::set_var("ALLOWED_ORIGINS", "invalid@url");
        let _cors = create_cors_layer();
        // Should fall back to permissive layer without panicking
    }

    #[test]
    fn test_cors_layer_whitespace() {
        env::set_var("ALLOWED_ORIGINS", "  *  ");
        let _cors = create_cors_layer();
        // Should handle whitespace and create Any origin layer
    }

    #[test]
    fn test_cors_layer_mixed_valid_invalid() {
        env::set_var("ALLOWED_ORIGINS", "https://example.com,invalid@url,http://localhost:3000");
        let _cors = create_cors_layer();
        // Should fall back to permissive layer due to invalid origin
    }
}
