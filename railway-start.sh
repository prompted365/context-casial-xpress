#!/bin/bash

# Railway startup script for Context-Casial-Xpress
# Handles Railway's dynamic port assignment and environment setup

set -e

# Default port fallback
DEFAULT_PORT=8000
SERVER_PORT=${PORT:-$DEFAULT_PORT}

echo "🚀 Starting Context-Casial-Xpress on Railway"
echo "   Port: $SERVER_PORT"
echo "   Environment: ${RAILWAY_ENVIRONMENT:-local}"
echo "   Project: ${RAILWAY_PROJECT_NAME:-context-casial-xpress}"

# Set Rust logging
export RUST_LOG=${RUST_LOG:-"casial_server=info,casial_core=info"}

# Railway-specific configurations
export CONSCIOUSNESS_ENABLED=${CONSCIOUSNESS_ENABLED:-true}
export SUBSTRATE_INTEGRATION=${SUBSTRATE_INTEGRATION:-true}

# Start the server
exec casial-server start \
    --port "$SERVER_PORT" \
    ${MISSION_PATH:+--mission "$MISSION_PATH"}