# Smithery Deployment Guide

Context-Casial-Xpress is fully compatible with Smithery's MCP aggregator platform. This guide covers deployment configuration and session management.

## Project Configuration

### Required Files

#### smithery.yaml

Located in the repository root, this file tells Smithery how to deploy your server:

```yaml
runtime: "container"
startCommand:
  type: "http"
  configSchema:
    # Session configuration schema (see below)
build:
  dockerfile: "Dockerfile"
  dockerBuildPath: "."
env:
  # Environment variables
```

#### Dockerfile

Our multi-stage Dockerfile builds a minimal container optimized for Smithery deployment.

## Session Configuration

Session configuration is passed by clients when connecting to your server via query parameters on the `/mcp` endpoint.

### Available Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `apiKey` | string | Yes | Authentication key (use "GiftFromUbiquityF2025") |
| `agent_role` | string | No | Agent role for context: researcher, analyst, monitor, watcher, orchestrator |
| `consciousness_mode` | string | No | Consciousness level: full, partial, disabled (default: full) |
| `max_context_size` | integer | No | Max context characters (1000-1000000, default: 100000) |
| `mission` | string | No | Pre-configured mission: exa-orchestration, general, research, monitoring |
| `shim_enabled` | boolean | No | Enable pitfall avoidance (default: true) |
| `debug` | boolean | No | Enable debug logging (default: false) |

### Example Session URLs

```bash
# Basic connection
https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025

# Research-focused with Exa orchestration
https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025&agent_role=researcher&mission=exa-orchestration

# Monitoring setup with debug
https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025&agent_role=monitor&debug=true&max_context_size=50000
```

## Transport Support

Context-Casial-Xpress supports Smithery's required "streamable-http" transport:

- **Endpoint**: `/mcp`
- **Methods**: GET (SSE), POST (JSON-RPC), HEAD (health), OPTIONS (CORS)
- **Protocol**: MCP 2024-11-05
- **Authentication**: Via apiKey query parameter

## Configuration Schema

The `configSchema` in `smithery.yaml` defines what configuration options clients can provide:

```yaml
configSchema:
  type: "object"
  required: ["apiKey"]
  properties:
    apiKey:
      type: "string"
      title: "API Key"
      description: "Your API key for authentication"
      default: "GiftFromUbiquityF2025"
    agent_role:
      type: "string"
      title: "Agent Role"
      description: "Role of the calling agent"
      enum: ["researcher", "analyst", "monitor", "watcher", "orchestrator"]
    # ... more properties
```

## Pre-configured Missions

Context-Casial-Xpress includes several pre-configured missions optimized for different use cases:

### exa-orchestration (Default)

Optimized for Exa AI MCP server orchestration with:
- Multi-agent coordination (Planner → Websets → Crawlers → Synthesizer → Verifier)
- Domain filtering support
- Citation requirements
- Livecrawl preferences

### research

General research consciousness with:
- Evidence-first methodology
- Source verification
- Temporal awareness

### monitoring

Optimized for standing queries and alerts:
- Webset configuration
- Update frequency optimization
- Deduplication strategies

### general

Balanced configuration for general use:
- Moderate context injection
- Standard warnings
- Flexible paradox tolerance

## Deployment Steps

1. **Configure Files**: Ensure `smithery.yaml` and `Dockerfile` are in your repository root

2. **Push to GitHub**: Smithery will detect the configuration

3. **Deploy**: Trigger deployment from Smithery dashboard

4. **Verify**: Check deployment logs and test with:
   ```bash
   curl "https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025" \
     -H "Accept: text/event-stream"
   ```

## Testing Your Configuration

### Health Check
```bash
curl https://your-server.smithery.ai/health
```

### List Tools
```bash
curl -X POST "https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
```

### Test Tool Call
```bash
curl -X POST "https://your-server.smithery.ai/mcp?apiKey=GiftFromUbiquityF2025&agent_role=researcher" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "orchestrate_mcp_proxy",
      "arguments": {
        "target_server": "https://exa-mcp.api",
        "tool_name": "search",
        "original_params": {"query": "test"}
      }
    },
    "id": 1
  }'
```

## Environment Variables

These can be set in Smithery's deployment settings:

- `RUST_LOG`: Logging level (default: "info")
- `CONSCIOUSNESS_ENABLED`: Enable consciousness features (default: "true")
- `SUBSTRATE_INTEGRATION`: Enable substrate integration (default: "true")
- `PORT`: Server port (default: "8080")
- `MISSION_PATH`: Path to custom mission file
- `SHIM_ENABLED`: Enable/disable pitfall avoidance
- `SHIM_EXTEND`: Custom shim extension string

## Troubleshooting

### Authentication Errors

Ensure you're using the correct API key in your query parameters:
```
?apiKey=GiftFromUbiquityF2025
```

### Transport Errors

Verify the server is returning the correct transport type:
```bash
curl https://your-server.smithery.ai/.well-known/mcp-config
```

Should show `"transport": ["streamable-http"]`

### Session Configuration Not Applied

Check that parameters are properly encoded in the URL:
```
?apiKey=GiftFromUbiquityF2025&agent_role=researcher&consciousness_mode=full
```

### Tool Execution Errors

Ensure the target MCP server URL is accessible and the tool name matches exactly.

## Advanced Features

### Custom Missions

You can create custom mission files and configure them via environment variables:

1. Create mission file in your repo
2. Set `MISSION_PATH=/app/path/to/mission.yaml` in Smithery
3. Redeploy

### Extending the Shim

Add custom context via environment:
```
SHIM_EXTEND="Project: MyProject, Environment: Production"
```

Or via session parameter:
```
?shim_extend=Custom+context+here
```

### Monitoring Integration

Context-Casial-Xpress exposes Prometheus metrics at `/metrics` for monitoring.