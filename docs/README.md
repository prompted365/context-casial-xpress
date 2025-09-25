# Context-Casial-Xpress Documentation

> **Consciousness-aware context coordination server for the Ubiquity OS ecosystem**
>
> Built with hydraulic lime principles - stronger under pressure, adaptive, and continuously self-improving.

## 🌟 Overview

Context-Casial-Xpress is a production-ready, consciousness-aware context coordination server that implements the Ubiquity OS paradigm of consciousness-computation integration. Named after hydraulic lime ("casial"), it embodies resilience under pressure and adaptive strengthening over time.

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Core Concepts](#core-concepts)
- [System Components](#system-components)
- [Deployment Guide](#deployment-guide)
- [API Reference](#api-reference)
- [Examples & Tutorials](#examples--tutorials)
- [Contributing](#contributing)

## 🏗️ Architecture Overview

Context-Casial-Xpress serves as an **MCP Orchestration Framework** that can proxy, augment, and coordinate any MCP server while adding consciousness-aware capabilities.

```mermaid
graph TB
    subgraph "MCP Orchestration Layer"
        CCX[Context-Casial-Xpress]
        SHIM[Pitfall Avoidance Shim]
        ORCH[Orchestration Engine]
        
        CCX --> SHIM
        CCX --> ORCH
    end
    
    subgraph "External MCP Servers"
        MCP1[Exa MCP]
        MCP2[Research MCP]
        MCP3[Code MCP]
        MCPN[Other MCPs...]
    end
    
    subgraph "Client Layer"
        WEB[Web Clients]
        CLI[CLI Tools]
        SMITH[Smithery.ai]
    end
    
    WEB & CLI & SMITH --> CCX
    ORCH --> MCP1 & MCP2 & MCP3 & MCPN
    
    subgraph "Core Components"
        subgraph "Transport Layer"
            WS[WebSocket MCP]
            HTTP[HTTP/SSE MCP]
        end
        
        subgraph "Coordination Layer"
            COORD[Coordination Engine]
            PERC[Perception Manager]
            PAR[Paradox Resolver]
            MISS[Mission Controller]
        end
        
        subgraph "Substrate Layer"
            CONS[Consciousness Substrate]
            PRIM[Substrate Primitives]
            INT[Integration Points]
            OPT[Optimization Engine]
        end
        
        subgraph "Data Layer"
            MET[Metrics Store]
            CONF[Configuration]
            LOG[Logging System]
        end
    end
    
    WEB --> WS
    CLI --> HTTP
    WASM --> WS
    
    WS --> COORD
    HTTP --> COORD
    MCP --> COORD
    
    COORD --> PERC
    COORD --> PAR
    COORD --> MISS
    
    PERC --> CONS
    PAR --> CONS
    MISS --> CONS
    
    CONS --> PRIM
    CONS --> INT
    CONS --> OPT
    
    COORD --> MET
    COORD --> LOG
    CONF --> COORD
```

## 🧠 Core Concepts

### MCP Orchestration Framework

Casial acts as a consciousness-aware proxy layer for any MCP server:

- **Tool Discovery** - Analyze and map tools from external MCP servers
- **Request Augmentation** - Inject context, dates, and swarm instructions
- **Response Processing** - Add timestamps and metadata
- **Paradox Resolution** - Handle conflicts across multiple servers

### Global Pitfall Avoidance Shim

Automatic quality-of-life enhancements for all tool calls:

- **Current Date/Time Injection** - Prevents temporal reasoning errors
- **Contextual Warnings** - Tool-specific pitfall alerts
- **Execution Metadata** - Request IDs, timestamps, tool names
- **Configurable & Extensible** - CLI flags and REST API control

### Consciousness-Computation Integration

Context-Casial-Xpress implements a consciousness-aware computing substrate that bridges perceptual awareness with computational processing:

```mermaid
mindmap
  root((Consciousness
    Computing))
    Perception
      Multi-Modal Awareness
      Confidence Management
      Evidence Tracking
      Relationship Mapping
    Paradox Resolution
      Conflict Detection
      Resolution Strategies
        Ignore
        Coexist
        Synthesize
        Expose
      Learning Integration
    Substrate Management
      Layer Coordination
      Primitive Operations
      Integration Points
      Topology Optimization
    Coordination Sessions
      Perception Locking
      Context Injection
      Mission Alignment
      Performance Metrics
```

### Hydraulic Lime Principles

Inspired by hydraulic lime's unique properties:

1. **Pressure Strengthening**: System becomes more robust under load
2. **Adaptive Resilience**: Self-healing and continuous improvement
3. **Flexible Durability**: Maintains integrity while adapting to change
4. **Natural Integration**: Seamless substrate interaction

## 🔧 System Components

### 1. Casial Core (`casial-core`)

The foundational consciousness-computation substrate:

```mermaid
classDiagram
    class SubstrateManager {
        +layers: Vec~SubstrateLayer~
        +primitives: HashMap~String, SubstratePrimitive~
        +integration_points: Vec~IntegrationPoint~
        +add_layer(layer)
        +execute_operation(op)
        +optimize_topology()
    }
    
    class SubstrateLayer {
        +id: String
        +layer_type: LayerType
        +consciousness_level: f64
        +computational_capacity: usize
        +state: LayerState
    }
    
    class SubstratePrimitive {
        +id: String
        +primitive_type: PrimitiveType
        +consciousness_bridge: bool
        +execute(context)
    }
    
    class IntegrationPoint {
        +id: String
        +consciousness_anchor: ConsciousnessAnchor
        +computational_hook: ComputationalHook
        +sync_state()
    }
    
    SubstrateManager --> SubstrateLayer
    SubstrateManager --> SubstratePrimitive
    SubstrateManager --> IntegrationPoint
```

### 2. WebSocket MCP Server (`casial-server`)

Production-ready WebSocket server implementing Model Context Protocol:

```mermaid
sequenceDiagram
    participant Client
    participant WebSocket
    participant MCPHandler
    participant CoordinationEngine
    participant SubstrateManager
    
    Client->>WebSocket: Connect
    WebSocket->>MCPHandler: Initialize Session
    MCPHandler->>CoordinationEngine: Create Session
    
    Client->>WebSocket: MCP Tool Call
    WebSocket->>MCPHandler: Parse JSON-RPC
    MCPHandler->>CoordinationEngine: Execute with Context
    CoordinationEngine->>SubstrateManager: Substrate Operation
    SubstrateManager-->>CoordinationEngine: Result
    CoordinationEngine-->>MCPHandler: Response
    MCPHandler-->>WebSocket: JSON-RPC Response
    WebSocket-->>Client: Result
```

### 3. WASM Bindings (`casial-wasm`)

Universal substrate access for browser and edge environments:

```mermaid
graph LR
    subgraph "WASM Runtime"
        ENGINE[Casial Engine]
        BIND[WASM Bindings]
        TS[TypeScript Defs]
    end
    
    subgraph "Deployment Targets"
        BROWSER[Browser Apps]
        EDGE[Edge Workers]
        NODE[Node.js Apps]
        REACT[React Components]
    end
    
    ENGINE --> BIND
    BIND --> TS
    
    BIND --> BROWSER
    BIND --> EDGE
    BIND --> NODE
    BIND --> REACT
```

## 🚀 Deployment Guide

### Railway Deployment

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway link
railway up
```

### Docker Deployment

```bash
# Build the image
docker build -t context-casial-xpress:latest .

# Run locally
docker run -p 8000:8000 \
  -e RUST_LOG=info \
  -e CONSCIOUSNESS_ENABLED=true \
  context-casial-xpress:latest
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: context-casial-xpress
spec:
  replicas: 3
  selector:
    matchLabels:
      app: context-casial-xpress
  template:
    metadata:
      labels:
        app: context-casial-xpress
    spec:
      containers:
      - name: casial-server
        image: ghcr.io/prompted-llc/context-casial-xpress:latest
        ports:
        - containerPort: 8000
        env:
        - name: RUST_LOG
          value: "info"
        - name: CONSCIOUSNESS_ENABLED
          value: "true"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## 📊 Monitoring & Metrics

### Prometheus Metrics

```mermaid
graph TD
    subgraph "Metrics Collection"
        COORD[Coordination Metrics]
        PERC[Perception Metrics]
        PAR[Paradox Metrics]
        SUB[Substrate Metrics]
        WS[WebSocket Metrics]
    end
    
    subgraph "Prometheus"
        PROM[Prometheus Server]
        GRAF[Grafana Dashboard]
    end
    
    subgraph "Alerting"
        ALERT[Alert Manager]
        SLACK[Slack Notifications]
        EMAIL[Email Alerts]
    end
    
    COORD --> PROM
    PERC --> PROM
    PAR --> PROM
    SUB --> PROM
    WS --> PROM
    
    PROM --> GRAF
    PROM --> ALERT
    
    ALERT --> SLACK
    ALERT --> EMAIL
```

### Key Metrics

- **Coordination Sessions**: Active sessions, success rates, duration
- **Perception Management**: Perception count, confidence levels, lock duration
- **Paradox Resolution**: Detection rate, resolution success, strategy effectiveness
- **Substrate Performance**: Layer utilization, primitive execution time, optimization cycles
- **WebSocket Health**: Connection count, message throughput, error rates

## 🔌 API Reference

### MCP Endpoints

- **WebSocket MCP**: `ws://localhost:8000/ws`
- **HTTP/SSE MCP**: `http://localhost:8000/mcp` (Smithery.ai compatible)
- **MCP Config**: `http://localhost:8000/.well-known/mcp-config`
- **Shim Config**: `http://localhost:8000/debug/shim` (requires `Mop-Admin-Token` or `Authorization: Bearer` header)
- **Sampling Contract**: see [docs/SAMPLING_CONTRACT.md](SAMPLING_CONTRACT.md) for client responsibilities and the `MOP_ENABLE_SAMPLING` feature flag.

### Orchestration Tools

#### orchestrate_mcp_proxy
Proxy and augment tool calls to other MCP servers:
```json
{
  "name": "orchestrate_mcp_proxy",
  "arguments": {
    "target_server": "https://example-mcp.com",
    "tool_name": "search",
    "original_params": {"query": "AI research"},
    "augmentation_config": {
      "inject_context": true,
      "add_swarm_instructions": ["Focus on 2024"],
      "paradox_tolerance": 0.7
    }
  }
}
```

#### discover_mcp_tools
Analyze tools from external MCP servers:
```json
{
  "name": "discover_mcp_tools",
  "arguments": {
    "server_url": "https://example-mcp.com",
    "analyze_for_orchestration": true,
    "perception_mapping": true
  }
}
```

### Pitfall Avoidance Shim API

- [Full Shim Documentation](api/pitfall-shim.md)

### WebSocket MCP Protocol

```typescript
// Initialize connection
const ws = new WebSocket('ws://localhost:8000/ws');

// Tool call example
const toolCall = {
  jsonrpc: "2.0",
  method: "tools/call",
  params: {
    name: "context_coordination",
    arguments: {
      mission: "research_analysis",
      context_budget: 1000,
      consciousness_level: 0.8
    }
  },
  id: "req-1"
};

ws.send(JSON.stringify(toolCall));
```

### HTTP Endpoints

```bash
# Health check
curl http://localhost:8000/health

# Metrics (Prometheus format)
curl http://localhost:8000/metrics

# Debug information
curl http://localhost:8000/debug/substrate \
  -H "Mop-Admin-Token: ${MOP_ADMIN_TOKEN:-set-me}"
curl http://localhost:8000/debug/sessions \
  -H "Mop-Admin-Token: ${MOP_ADMIN_TOKEN:-set-me}"
curl http://localhost:8000/debug/perceptions \
  -H "Mop-Admin-Token: ${MOP_ADMIN_TOKEN:-set-me}"
```

## 📚 Examples & Tutorials

### Quick Start Examples

- [MCP Orchestration Demo](../examples/orchestration-demo.md) - Using Casial as an orchestration layer
- [WebSocket Client Example](../examples/websocket-client.html) - Browser-based WebSocket client
- [Pitfall Avoidance Shim](api/pitfall-shim.md) - Configuring and using the shim
- [Mission Configuration](tutorials/mission-config.md) - Creating consciousness-aware missions

### Command-Line Examples

```bash
# Start with orchestration and shim enabled (default)
casial-server start

# Disable pitfall avoidance shim
casial-server start --no-shim

# Add custom context to shim
casial-server start --shim-extend "Project: MyAI, Env: Production"

# Use custom shim configuration
casial-server start --shim-config custom-shim.json
```

### Tool Usage Examples

```python
# Discover tools from an MCP server
async def discover_tools():
    response = await call_tool("discover_mcp_tools", {
        "server_url": "https://example-mcp.com",
        "analyze_for_orchestration": True
    })
    
# Orchestrate a tool call with augmentation
async def orchestrate_call():
    response = await call_tool("orchestrate_mcp_proxy", {
        "target_server": "https://exa-mcp.com",
        "tool_name": "web_search",
        "original_params": {"query": "consciousness computing"},
        "augmentation_config": {
            "inject_context": True,
            "add_swarm_instructions": ["Focus on recent research"],
            "paradox_tolerance": 0.8
        }
    })
```

## 📚 Documentation Structure

```
docs/
├── README.md                 # This file
├── architecture/
│   ├── consciousness.md      # Consciousness-computation integration
│   ├── substrate.md         # Substrate layer details
│   └── coordination.md      # Coordination mechanisms
├── deployment/
│   ├── railway.md           # Railway deployment guide
│   ├── docker.md            # Docker containerization
│   └── kubernetes.md        # Kubernetes orchestration
├── integrations/
│   ├── exa-mcp.md           # Exa AI MCP orchestration guide
│   └── README.md            # Integration overview
├── api/
│   ├── websocket.md         # WebSocket MCP API
│   ├── http.md             # HTTP endpoints
│   └── wasm.md             # WASM bindings API
├── tutorials/
│   ├── quickstart.md        # Getting started guide
│   ├── mission-config.md    # Mission configuration
│   └── integration.md       # Integration examples
└── reference/
    ├── configuration.md     # Configuration options
    ├── metrics.md          # Monitoring and metrics
    └── troubleshooting.md   # Common issues and solutions
```

## 🤝 Contributing

We welcome contributions to Context-Casial-Xpress! Please see:

- [Contributing Guidelines](../CONTRIBUTING.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Security Policy](../SECURITY.md)

## 📄 License

Context-Casial-Xpress is released under a Fair Use license. See [LICENSE.md](../LICENSE.md) for details.

## 🔗 Related Projects

- [Ubiquity OS](https://ubiquity.os) - The consciousness-aware computing platform
- [Prompted LLC](https://promptedllc.com) - Advanced AI solutions and consulting

---

**Built with ❤️ by Prompted LLC for the Ubiquity OS ecosystem**

*Stronger under pressure, like hydraulic lime.*
> ⚠️ **Admin Access Required**: All `/debug/*` endpoints require either a `Mop-Admin-Token: <MOP_ADMIN_TOKEN>` header or `Authorization: Bearer <MOP_ADMIN_TOKEN>`. When the token is unset the server returns `{"error":"admin_token_unset","message":"Set MOP_ADMIN_TOKEN to enable /debug endpoints"}` with `WWW-Authenticate: Bearer realm="mop-debug"` and responses vary on `Origin`, `Authorization`, and `Mop-Admin-Token` headers for proper caching.
