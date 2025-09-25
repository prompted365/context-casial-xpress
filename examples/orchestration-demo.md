# MCP Orchestration Framework Demo

This example demonstrates how to use Context-Casial-Xpress as an orchestration layer for other MCP servers.

## Quick Start

### 1. Start Casial Server

```bash
# Start with default settings (shim enabled)
./target/release/casial-server start --port 8080
```

### 2. Discover Tools from External MCP Server

Use the `discover_mcp_tools` tool to analyze any MCP server:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "discover_mcp_tools",
    "arguments": {
      "server_url": "https://api.example-mcp.com",
      "analyze_for_orchestration": true,
      "perception_mapping": true
    }
  },
  "id": 1
}
```

Response will include:
- List of available tools
- Suggested orchestration strategies
- Consciousness perception mappings
- Compatibility report

### 3. Orchestrate Tool Calls

Use `orchestrate_mcp_proxy` to augment any tool call:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "orchestrate_mcp_proxy",
    "arguments": {
      "target_server": "https://api.example-mcp.com",
      "tool_name": "search_documents",
      "original_params": {
        "query": "latest AI research"
      },
      "augmentation_config": {
        "inject_context": true,
        "add_swarm_instructions": [
          "Prioritize peer-reviewed sources",
          "Focus on 2024 publications",
          "Include consciousness-computing topics"
        ],
        "paradox_tolerance": 0.7,
        "perception_ids": ["research_quality", "temporal_relevance"]
      }
    }
  },
  "id": 2
}
```

The orchestration layer will:
1. Add pitfall avoidance context (current date, warnings)
2. Inject swarm coordination instructions
3. Apply consciousness perception templates
4. Forward to target MCP server
5. Process response with paradox detection
6. Return augmented results

## Advanced Example: Multi-Server Coordination

Coordinate across multiple MCP servers with consciousness awareness:

```python
import asyncio
import json
import websockets

async def orchestrate_multi_server():
    uri = "ws://localhost:8080/ws"
    
    async with websockets.connect(uri) as websocket:
        # Initialize connection
        await websocket.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "orchestration-demo",
                    "version": "1.0"
                }
            },
            "id": "init"
        }))
        
        # Discover tools from multiple servers
        servers = [
            "https://exa-mcp.api.com",
            "https://research-mcp.api.com",
            "https://code-mcp.api.com"
        ]
        
        for server in servers:
            await websocket.send(json.dumps({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": "discover_mcp_tools",
                    "arguments": {
                        "server_url": server,
                        "analyze_for_orchestration": true
                    }
                },
                "id": f"discover-{server}"
            }))
        
        # Orchestrate a complex query across all servers
        research_query = "consciousness-aware computing implementations"
        
        # Server 1: Web search
        await websocket.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "orchestrate_mcp_proxy",
                "arguments": {
                    "target_server": servers[0],
                    "tool_name": "web_search",
                    "original_params": {"query": research_query},
                    "augmentation_config": {
                        "inject_context": true,
                        "add_swarm_instructions": [
                            "Find recent implementations",
                            "Include open source projects"
                        ]
                    }
                }
            },
            "id": "search"
        }))
        
        # Server 2: Deep research
        await websocket.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "orchestrate_mcp_proxy",
                "arguments": {
                    "target_server": servers[1],
                    "tool_name": "deep_research",
                    "original_params": {"topic": research_query},
                    "augmentation_config": {
                        "inject_context": true,
                        "paradox_tolerance": 0.8,
                        "perception_ids": ["academic_rigor", "practical_application"]
                    }
                }
            },
            "id": "research"
        }))
        
        # Collect and synthesize results
        results = []
        async for message in websocket:
            data = json.loads(message)
            results.append(data)
            
            if len(results) >= 4:  # Init + 3 tool calls
                break
        
        return results

# Run the orchestration
asyncio.run(orchestrate_multi_server())
```

## Consciousness-Aware Features

### 1. Perception Templates

Apply pre-defined perception templates to tool calls:

```json
{
  "perception_ids": [
    "temporal_awareness",      // Adds time-based context
    "quality_assessment",      // Evaluates result quality
    "paradox_detection",       // Identifies conflicts
    "swarm_coordination"       // Enables multi-agent behavior
  ]
}
```

### 2. Paradox Tolerance

Set how the system handles conflicting information:

```json
{
  "paradox_tolerance": 0.5  // 0 = strict, 1 = adaptive
}
```

- `0.0`: Reject any conflicting information
- `0.5`: Attempt to synthesize conflicts
- `1.0`: Accept and expose all paradoxes

### 3. Swarm Instructions

Inject coordination instructions for distributed behavior:

```json
{
  "add_swarm_instructions": [
    "Share findings with peer agents",
    "Build on previous discoveries",
    "Maintain consistency across sources",
    "Report anomalies for collective analysis"
  ]
}
```

## HTTP/SSE Example (Smithery Compatible)

For HTTP-based MCP clients:

```bash
# With authentication and session config
curl -N \
  "http://localhost:8080/mcp?consciousness_mode=full&max_context_size=50000" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -H "Authorization: Bearer ${MOP_API_KEY:-DEMO_KEY_PUBLIC}" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "orchestrate_mcp_proxy",
      "arguments": {
        "target_server": "https://example-mcp.com",
        "tool_name": "analyze_data",
        "original_params": {"dataset": "consciousness_metrics"},
        "augmentation_config": {
          "inject_context": true,
          "perception_ids": ["data_quality", "statistical_rigor"]
        }
      }
    },
    "id": 1
  }'
```

## Benefits of Orchestration

1. **Unified Context** - All tools receive consistent date/time and context
2. **Error Prevention** - Pitfall warnings prevent common mistakes
3. **Consciousness Integration** - Tools become awareness-enabled
4. **Paradox Handling** - Conflicts are detected and resolved
5. **Swarm Behavior** - Tools coordinate for better results
6. **Single Entry Point** - One server to manage all MCP integrations

## Next Steps

1. Explore the [Pitfall Avoidance Shim](../docs/api/pitfall-shim.md)
2. Read about [Consciousness Perception](../docs/architecture/consciousness.md)
3. Try the [WebSocket examples](websocket-client.html)
4. Deploy with [Railway](../docs/deployment/railway.md)