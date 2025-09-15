# Quick Start Guide: Context-Casial-Xpress

> **From zero to consciousness-aware coordination in 5 minutes**

This guide walks you through deploying Context-Casial-Xpress from source to production, demonstrating the consciousness-computation substrate in action.

## 🚀 Prerequisites

- **Rust 1.80+** with `cargo` and `wasm-pack`
- **Node.js 18+** for WASM package management
- **Docker** for containerized deployment (optional)
- **Railway CLI** for production deployment (optional)

## 📦 Installation

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/prompted-llc/context-casial-xpress.git
cd context-casial-xpress

# Build all crates
cargo build --release

# Build WASM bindings
wasm-pack build crates/casial-wasm --target web
```

### 2. Configuration Setup

Create a basic configuration file:

```yaml
# config.yaml
server:
  host: "0.0.0.0"
  port: 8000
  websocket_path: "/ws"

consciousness:
  enabled: true
  substrate_integration: true
  perception_confidence_threshold: 0.7
  paradox_detection_sensitivity: 0.8

logging:
  level: "info"
  format: "json"

metrics:
  enabled: true
  prometheus_endpoint: "/metrics"
```

### 3. Create Example Mission

```yaml
# missions/research_mission.yaml
mission:
  name: "research_analysis"
  description: "Consciousness-aware research and analysis coordination"
  
goals:
  primary:
    - "Coordinate multi-source research tasks"
    - "Resolve conflicting information through paradox resolution"
    - "Maintain perceptual awareness of research context"
  
context_budget: 2000
consciousness_level: 0.8

coordination:
  perception_locking: true
  paradox_resolution: "synthesize"
  context_injection_cadence: "adaptive"

templates:
  research_context: |
    Research Mission: {mission_name}
    Current Phase: {research_phase}
    Sources: {active_sources}
    Paradoxes Detected: {paradox_count}
    Confidence Level: {overall_confidence}
```

## 🎯 Running the Server

### Development Mode

```bash
# Start with debug logging
RUST_LOG=debug cargo run --bin casial-server -- start --config config.yaml --dev

# Server starts at http://localhost:8000
# WebSocket endpoint: ws://localhost:8000/ws
# Health check: http://localhost:8000/health
# Metrics: http://localhost:8000/metrics
```

### Production Mode

```bash
# Build release binary
cargo build --release

# Run production server
./target/release/casial-server start --config config.yaml --port 8000
```

## 🔌 Testing the WebSocket Connection

### Using `websocat` (Recommended)

```bash
# Install websocat
cargo install websocat

# Connect to the server
websocat ws://localhost:8000/ws

# Send MCP initialization
{"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}}, "id": 1}

# Example tool call
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "context_coordination",
    "arguments": {
      "mission": "research_analysis",
      "context": "Analyze market trends for AI infrastructure",
      "consciousness_level": 0.8
    }
  },
  "id": 2
}
```

### Using JavaScript/Browser

```html
<!DOCTYPE html>
<html>
<head>
    <title>Casial WebSocket Test</title>
</head>
<body>
    <script>
        const ws = new WebSocket('ws://localhost:8000/ws');
        
        ws.onopen = function(event) {
            console.log('Connected to Casial server');
            
            // Initialize MCP session
            ws.send(JSON.stringify({
                jsonrpc: "2.0",
                method: "initialize",
                params: { capabilities: {} },
                id: 1
            }));
        };
        
        ws.onmessage = function(event) {
            const response = JSON.parse(event.data);
            console.log('Received:', response);
            
            // After initialization, call a tool
            if (response.id === 1) {
                ws.send(JSON.stringify({
                    jsonrpc: "2.0",
                    method: "tools/call",
                    params: {
                        name: "context_coordination",
                        arguments: {
                            mission: "research_analysis",
                            query: "What are the latest trends in AI infrastructure?",
                            consciousness_level: 0.8
                        }
                    },
                    id: 2
                }));
            }
        };
        
        ws.onerror = function(error) {
            console.error('WebSocket error:', error);
        };
        
        ws.onclose = function(event) {
            console.log('Connection closed');
        };
    </script>
</body>
</html>
```

## 🐳 Docker Deployment

### Build and Run

```bash
# Build the Docker image
docker build -t context-casial-xpress:latest .

# Run with environment variables
docker run -p 8000:8000 \
  -e RUST_LOG=info \
  -e CONSCIOUSNESS_ENABLED=true \
  -e SUBSTRATE_INTEGRATION=true \
  -v $(pwd)/config.yaml:/app/config.yaml \
  -v $(pwd)/missions:/app/missions \
  context-casial-xpress:latest
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  casial-server:
    build: .
    ports:
      - "8000:8000"
    environment:
      - RUST_LOG=info
      - CONSCIOUSNESS_ENABLED=true
      - SUBSTRATE_INTEGRATION=true
    volumes:
      - ./config.yaml:/app/config.yaml
      - ./missions:/app/missions
      - casial-data:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

volumes:
  casial-data:
```

## 🚅 Railway Deployment

### Setup Railway Project

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login to Railway
railway login

# Create new project
railway create context-casial-xpress

# Add environment variables
railway env set RUST_LOG=info
railway env set CONSCIOUSNESS_ENABLED=true
railway env set SUBSTRATE_INTEGRATION=true
railway env set PORT=8000

# Deploy
railway up
```

### railway.toml Configuration

```toml
[build]
builder = "dockerfile"

[deploy]
healthcheck_endpoint = "/health"
healthcheck_timeout = 300

[env]
RUST_LOG = "info"
CONSCIOUSNESS_ENABLED = "true"
SUBSTRATE_INTEGRATION = "true"
PORT = "8000"
```

## 📊 Monitoring Your Deployment

### Health Check

```bash
# Basic health check
curl http://localhost:8000/health

# Expected response:
# {"status":"healthy","consciousness_active":true,"substrate_integrated":true}
```

### Prometheus Metrics

```bash
# View metrics
curl http://localhost:8000/metrics

# Key metrics to monitor:
# - casial_coordination_sessions_active
# - casial_perception_locks_held
# - casial_paradox_resolutions_total
# - casial_websocket_connections_active
```

### Debug Endpoints

```bash
# Substrate status
curl http://localhost:8000/debug/substrate

# Active sessions
curl http://localhost:8000/debug/sessions

# Perception state
curl http://localhost:8000/debug/perceptions
```

## 🧪 Testing Consciousness Features

### 1. Perception Management Test

```bash
# Create a perception
curl -X POST http://localhost:8000/debug/perceptions \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Market analysis indicates strong AI infrastructure growth",
    "confidence": 0.85,
    "evidence": ["Report A", "Survey B", "Expert Opinion C"]
  }'

# Lock the perception
curl -X POST http://localhost:8000/debug/perceptions/lock \
  -H "Content-Type: application/json" \
  -d '{"perception_id": "generated-id", "session_id": "test-session"}'
```

### 2. Paradox Resolution Test

```bash
# Inject conflicting perceptions
curl -X POST http://localhost:8000/debug/paradox \
  -H "Content-Type: application/json" \
  -d '{
    "perception_a": {
      "content": "AI infrastructure market will grow 50%",
      "confidence": 0.8
    },
    "perception_b": {
      "content": "AI infrastructure market will decline 20%",
      "confidence": 0.7
    }
  }'

# Check resolution strategy
curl http://localhost:8000/debug/paradox/latest
```

## 🎓 Next Steps

### 1. Explore Advanced Features
- **Mission Configuration**: Create complex multi-phase missions
- **WASM Integration**: Deploy consciousness to edge environments  
- **Custom Tools**: Build consciousness-aware tool integrations
- **Monitoring Setup**: Deploy with Grafana dashboards

### 2. Production Considerations
- **Load Testing**: Validate 10k+ concurrent connections
- **Security**: Configure TLS and authentication
- **Scaling**: Multi-instance deployment with shared substrate
- **Backup**: Mission and perception state persistence

### 3. Integration Examples
- **AI Tool Coordination**: Connect multiple LLMs through MCP
- **Real-time Analytics**: Stream consciousness metrics to dashboards
- **Edge Deployment**: WASM bindings for browser/worker environments
- **Enterprise Integration**: Custom protocols and enterprise features

## 🆘 Troubleshooting

### Common Issues

**Connection Refused**
```bash
# Check if server is running
ss -tlnp | grep 8000

# Check logs
tail -f /app/logs/casial-server.log
```

**WebSocket Handshake Failed**
```bash
# Verify WebSocket path
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: test" \
  http://localhost:8000/ws
```

**Consciousness Features Not Working**
```bash
# Check configuration
grep -E "consciousness|substrate" config.yaml

# Verify environment
echo $CONSCIOUSNESS_ENABLED $SUBSTRATE_INTEGRATION
```

### Getting Help

- **Documentation**: [docs.contextcasial.ubiquity.os](https://docs.contextcasial.ubiquity.os)
- **Discord**: [discord.gg/ubiquity-os](https://discord.gg/ubiquity-os)  
- **GitHub Issues**: [github.com/prompted-llc/context-casial-xpress/issues](https://github.com/prompted-llc/context-casial-xpress/issues)
- **Email Support**: engineering@promptedllc.com

---

**🎉 Congratulations!** You now have a consciousness-aware context coordination server running in production. 

*Built stronger under pressure, like hydraulic lime* 🏗️