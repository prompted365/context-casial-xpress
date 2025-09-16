#!/bin/bash

# Railway startup script for Meta-Orchestration Protocol (MOP)
# Handles Railway's dynamic port assignment and environment setup
# Supports Smithery session configuration

set -e

# Default port fallback
DEFAULT_PORT=8080
SERVER_PORT=${PORT:-$DEFAULT_PORT}

echo "🚀 Starting Meta-Orchestration Protocol (MOP) on Railway"
echo "   Port: $SERVER_PORT"
echo "   Environment: ${RAILWAY_ENVIRONMENT:-local}"
echo "   Project: ${RAILWAY_PROJECT_NAME:-meta-orchestration-protocol}"
echo "   Smithery Compatible: ✅"

# Set Rust logging
export RUST_LOG=${RUST_LOG:-"casial_server=info,casial_core=info"}

# Railway-specific configurations
export CONSCIOUSNESS_ENABLED=${CONSCIOUSNESS_ENABLED:-true}
export SUBSTRATE_INTEGRATION=${SUBSTRATE_INTEGRATION:-true}

# Handle mission configuration
# Default to Exa orchestration if no mission specified
DEFAULT_MISSION="/app/missions/exa-mcp-orchestration.yaml"
if [ -z "$MISSION_PATH" ]; then
    if [ -f "$DEFAULT_MISSION" ]; then
        MISSION_PATH="$DEFAULT_MISSION"
        echo "   Mission: Using default Exa MCP orchestration"
    elif [ -f "/app/examples/ubiquity-mission.yaml" ]; then
        MISSION_PATH="/app/examples/ubiquity-mission.yaml"
        echo "   Mission: Using Ubiquity mission fallback"
    fi
fi

# Handle shim configuration from environment
SHIM_FLAGS=""
if [ "$SHIM_ENABLED" = "false" ]; then
    SHIM_FLAGS="--no-shim"
elif [ -n "$SHIM_EXTEND" ]; then
    SHIM_FLAGS="--shim-extend \"$SHIM_EXTEND\""
fi

# Start the server with all configurations
echo "   Starting with configuration:"
echo "   - HTTP/SSE MCP: http://0.0.0.0:$SERVER_PORT/mcp"
echo "   - WebSocket MCP: ws://0.0.0.0:$SERVER_PORT/ws"
echo "   - Health: http://0.0.0.0:$SERVER_PORT/health"

exec casial-server start \
    --port "$SERVER_PORT" \
    ${MISSION_PATH:+--mission "$MISSION_PATH"} \
    $SHIM_FLAGS