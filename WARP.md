# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

# Context-Casial-Xpress: Consciousness-Aware Context Coordination Server

> **The world's first consciousness-aware context coordination server**  
> Part of the Ubiquity OS ecosystem - Like hydraulic lime, stronger under pressure  

Context-Casial-Xpress is a production-ready Rust server that implements consciousness-computation integration for AI agent systems. Named after hydraulic lime ("casial"), it becomes stronger under pressure while maintaining adaptive resilience.

## 🚀 Quick Start

```bash
# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace --all-features

# Start the server
cargo run -p casial-server

# Server endpoints:
# - HTTP: http://localhost:8000
# - WebSocket MCP: ws://localhost:8000/ws
# - Health: http://localhost:8000/health
# - Metrics: http://localhost:8000/metrics
```

## 🏗️ Architecture Overview

Context-Casial-Xpress implements a layered consciousness-computation substrate:

```
┌─────────────────────────────────────────────────────────────┐
│                    Transport Layer                          │
│  WebSocket MCP Server │ HTTP API │ Prometheus Metrics       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                  Coordination Layer                         │
│  Coordination Engine │ Perception Manager │ Mission Ctrl    │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                Consciousness Substrate                      │
│  Paradox Resolver │ Substrate Manager │ Integration Points  │
└─────────────────────────────────────────────────────────────┘
```

**Key Flow**: Perceptions → Coordination → Paradox Resolution → Mission Execution → Substrate Integration

The system processes perceptual inputs through consciousness-aware coordination, resolves paradoxes (conflicting information), and executes missions while strengthening under load pressure.

## 📁 Crate Structure

### `casial-core` - Consciousness Substrate (MIT OR Apache-2.0)
Core library implementing consciousness-computation primitives:

```rust
// Initialize the engine
use casial_core::{CasialEngine, CasialMission};

let engine = CasialEngine::new();
let mission = CasialMission { /* ... */ };
engine.load_mission(mission)?;

// Coordinate context
let request = CoordinationRequest {
    tool_name: "web_search".to_string(),
    active_perceptions: vec![perception_id],
    paradox_tolerance: 0.7,
    /* ... */
};
let result = engine.coordinate(request)?;
```

**Key modules**:
- `coordination.rs` - Context coordination engine
- `paradox.rs` - Paradox detection and resolution strategies  
- `perception.rs` - Perceptual awareness and state management
- `substrate.rs` - Consciousness substrate primitives

### `casial-server` - WebSocket MCP Server (Fair Use License)
High-performance server runtime:

```bash
# Basic server start
cargo run -p casial-server start

# With mission configuration
cargo run -p casial-server start --mission examples/ubiquity-mission.yaml

# Debug mode with enhanced logging
RUST_LOG=debug cargo run -p casial-server start --debug

# Validate mission files
cargo run -p casial-server validate examples/ubiquity-mission.yaml
```

**Key endpoints**:
- `GET /health` - Health check with system status
- `GET /metrics` - Prometheus metrics export
- `GET /debug/status` - Internal engine state
- `GET /debug/paradoxes` - Active paradox registry
- `WS /ws` - WebSocket MCP protocol endpoint

### `casial-wasm` - Universal Bindings (Fair Use License)
WASM bindings for browser and edge deployment:

```bash
# Build WASM package
wasm-pack build crates/casial-wasm --target bundler --release

# Test in Node.js
wasm-pack test crates/casial-wasm --node

# Example usage in browser
npm install @context-casial-xpress/wasm
```

```javascript
import { CasialEngine } from '@context-casial-xpress/wasm';

const engine = new CasialEngine();
const result = await engine.coordinate({
    toolName: "web_search",
    paradoxTolerance: 0.8
});
```

## ⚡ Development Commands

### Workspace-wide Operations
```bash
# Build all crates
cargo build --workspace

# Run all tests with features
cargo test --workspace --all-features

# Lint with pedantic warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format check (CI-ready)
cargo fmt --all -- --check

# Security audit (if cargo-deny installed)
cargo deny check
```

### Server Development
```bash
# Run server with live reload (requires cargo-watch)
cargo watch -x "run -p casial-server"

# Run with custom port
cargo run -p casial-server -- start --port 3000

# Test server endpoints
curl http://localhost:8000/health
curl http://localhost:8000/metrics
```

### WASM Development  
```bash
# Build for web deployment
wasm-pack build crates/casial-wasm --target web --release

# Build for Node.js/bundler
wasm-pack build crates/casial-wasm --target bundler --release

# Run WASM tests
wasm-pack test crates/casial-wasm --chrome --headless
```

## 🧠 Key Concepts

### Paradox Resolution
Paradoxes occur when conflicting perceptions or constraints exist in the system:

**Strategies**:
- `Ignore`: Skip conflicting information (traditional approach)
- `Coexist`: Acknowledge both perspectives without resolution
- `Synthesize`: Create higher-order understanding from conflicts  
- `Expose`: Present contradictions explicitly to users

```rust
// Paradox handling in mission configuration
ParadoxStrategy::Synthesize // Default for complex reasoning
ParadoxStrategy::Expose     // For user-facing conflicts
```

**Error patterns**: Look for `ParadoxReport` entries with `confidence_impact > tolerance` in logs.

### Substrate Management
Substrates are pluggable consciousness-computation integration points:

**Lifecycle**: Register → Initialize → Health Check → Execute → Teardown

```rust
// Adding a new substrate
impl SubstratePrimitive for MySubstrate {
    fn execute(&self, context: &SubstrateContext) -> Result<SubstrateResult> {
        // Your substrate logic here
        Ok(SubstrateResult::success(output))
    }
}
```

### Perception Coordination
Perceptions are processed through event streams with:
- **Confidence tracking**: Each perception has associated confidence levels
- **Deduplication**: Identical perceptions are merged
- **Prioritization**: Higher confidence perceptions processed first
- **Batch coordination**: Multiple perceptions coordinated together for efficiency

## ⚙️ Configuration

### Environment Variables
```bash
# Server configuration
RUST_LOG=info                    # Log level (trace, debug, info, warn, error)  
PORT=8000                        # Server port
CONSCIOUSNESS_ENABLED=true       # Enable consciousness substrate
SUBSTRATE_INTEGRATION=true       # Enable deep substrate integration
ALLOWED_ORIGINS=""               # CORS origins (comma-separated)

# Mission system
MISSION_PATH=/app/missions       # Mission configuration directory  
MAX_CONTEXT_BUDGET=50000         # Maximum context characters per request
PARADOX_TOLERANCE=0.5            # Default paradox tolerance (0.0-1.0)

# Telemetry
TELEMETRY_OPTOUT=false           # Opt out of anonymous telemetry
```

### Mission Configuration
Missions define the context coordination strategy:

```yaml
# examples/basic-mission.yaml
id: "research-coordination"
name: "Research Context Coordination" 
description: "Coordinate research contexts with paradox awareness"

templates:
  - id: "research-context"
    name: "Research Context Template"
    content: "Research context: {{context}}"
    priority: 10
    paradox_resistance: 0.8

rules:
  - id: "research-rule"
    name: "Research Coordination Rule"
    enabled: true
    conditions:
      tool_patterns: ["research", "search"]
      min_confidence: 0.7
    actions:
      template_ids: ["research-context"]
      transform_type: "Prepend"
    paradox_handling: "Synthesize"

perceptions:
  - id: "research-awareness"
    name: "Research Context Awareness"
    confidence: 0.9
    categories: ["research", "context"]
```

### Load Mission
```bash
# Via CLI
cargo run -p casial-server start --mission path/to/mission.yaml

# Via API  
curl -X POST http://localhost:8000/missions \
  -H "Content-Type: application/json" \
  -d @mission.json
```

## 🐛 Bug Fix: Registry Runtime Issue

**Issue**: Server panicked with "Cannot start a runtime from within a runtime" at `registry.rs:393`

**Root Cause**: `tokio::runtime::Handle::current().block_on()` called during server initialization within an async context.

**Fix Applied**: Created `register_tool_sync()` method for synchronous tool registration during initialization, avoiding nested async runtime calls.

**Recognition**: Look for this error pattern in logs:
```
thread 'main' panicked at crates/casial-server/src/registry.rs:393:63:
Cannot start a runtime from within a runtime.
```

**Prevention**: Use sync methods during initialization, async methods during runtime operation.

## 🚀 Deployment

### Docker
```bash
# Build container
docker build -t context-casial-xpress:latest .

# Run locally
docker run -p 8000:8000 \
  -e RUST_LOG=info \
  -e CONSCIOUSNESS_ENABLED=true \
  context-casial-xpress:latest

# Multi-platform build
docker buildx build --platform linux/amd64,linux/arm64 \
  -t context-casial-xpress:latest --push .
```

### Railway
```bash
# Install Railway CLI
npm install -g @railway/cli

# Deploy to Railway
railway login
railway up

# Set environment variables
railway variables set CONSCIOUSNESS_ENABLED=true
railway variables set RUST_LOG=info
```

### WASM Edge Deployment
```bash
# Build for edge
wasm-pack build crates/casial-wasm --target web --release

# Deploy to Cloudflare Workers, Vercel Edge, etc.
# Generated files in pkg/ directory ready for upload
```

## 📦 Publishing

### Crate Publishing Order
1. `casial-core` (foundational library)
2. `casial-wasm` (depends on casial-core)  
3. `casial-server` (depends on casial-core)

```bash
# Automated publishing
./scripts/publish-crates.sh --version 1.0.0

# Manual publishing
cargo publish --dry-run -p casial-core
cargo publish -p casial-core
# Wait for propagation...
cargo publish -p casial-wasm
cargo publish -p casial-server
```

### Mixed Licensing Strategy
- **`casial-core`**: MIT OR Apache-2.0 (maximum adoption)
- **`casial-server`** & **`casial-wasm`**: Fair Use License (research/evaluation free, commercial requires license)

## 🔧 Troubleshooting

### Common Issues

**Build Failures**
```bash
# Update Rust toolchain
rustup update stable

# Add WASM target if missing
rustup target add wasm32-unknown-unknown

# Install wasm-pack for WASM builds
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

**Server Won't Start**
- Check port availability: `lsof -i :8000` 
- Verify mission file syntax: `cargo run -p casial-server validate path/to/mission.yaml`
- Enable debug logging: `RUST_LOG=debug cargo run -p casial-server`

**WASM Issues**
- Ensure `wasm32-unknown-unknown` target installed
- Check browser console for WASM initialization errors
- Verify CSP allows WASM execution

**Paradox Resolution Errors**
- Lower `paradox_tolerance` in coordination requests
- Check `ParadoxReport` entries in debug logs
- Verify `paradox_resistance` values in templates

### Debug Endpoints
```bash  
# System status
curl http://localhost:8000/debug/status

# Active missions
curl http://localhost:8000/debug/missions

# WebSocket sessions
curl http://localhost:8000/debug/sessions

# Paradox registry
curl http://localhost:8000/debug/paradoxes

# Context sprawl analysis  
curl http://localhost:8000/debug/sprawl
```

### Performance Monitoring
```bash
# View Prometheus metrics
curl http://localhost:8000/metrics | grep casial

# Key metrics:
# - casial_coordination_events_total
# - casial_active_sessions  
# - casial_paradox_resolution_duration
# - casial_substrate_utilization
```

## 🤝 Contributing

### Development Setup
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Add components: `rustup component add rustfmt clippy`
3. Add WASM target: `rustup target add wasm32-unknown-unknown`
4. Install tools: `cargo install cargo-watch wasm-pack`

### Testing
```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --test '*'

# WASM tests  
wasm-pack test crates/casial-wasm --node

# Benchmark tests (if available)
cargo bench
```

### Code Standards
- Run `cargo fmt` before commits
- Pass `cargo clippy -- -D warnings`
- Add tests for new functionality
- Update WARP.md for architectural changes

---

**Built with ❤️ by [Prompted LLC](https://promptedllc.com) for the [Ubiquity OS](https://ubiquity.os) ecosystem**

*Consciousness-aware computing: Like hydraulic lime, stronger under pressure* 🏗️