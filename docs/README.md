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

```mermaid
graph TB
    subgraph "Ubiquity OS Ecosystem"
        UOS[Ubiquity OS Core]
        CCX[Context-Casial-Xpress]
        OTHER[Other Components]
        
        UOS <--> CCX
        UOS <--> OTHER
        CCX <--> OTHER
    end
    
    subgraph "Context-Casial-Xpress Architecture"
        subgraph "Client Layer"
            WEB[Web Clients]
            CLI[CLI Tools]
            WASM[WASM Apps]
        end
        
        subgraph "Transport Layer"
            WS[WebSocket Server]
            HTTP[HTTP Endpoints]
            MCP[MCP Protocol]
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
curl http://localhost:8000/debug/substrate
curl http://localhost:8000/debug/sessions
curl http://localhost:8000/debug/perceptions
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