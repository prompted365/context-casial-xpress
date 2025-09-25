# Exa MCP Integration Guide

Context-Casial-Xpress provides turnkey support for orchestrating Exa AI's MCP tools with consciousness-aware augmentation. This guide shows how to leverage the built-in Exa templates and orchestration capabilities.

## Quick Start

### 1. Start Server with Exa Mission

```bash
# Start with Exa orchestration enabled
casial-server start --mission missions/exa-mcp-orchestration.yaml

# Or with custom settings
casial-server start \
  --mission missions/exa-mcp-orchestration.yaml \
  --shim-extend "Project: MyResearch, Mode: Production"
```

### 2. Configure Your MCP Client

```json
{
  "servers": {
    "casial-exa-orchestrator": {
      "transport": ["streamable-http", "websocket"],
      "url": "http://localhost:8080/mcp",
      "config": {
        "apiKey": "REPLACE_WITH_YOUR_API_KEY",
        "agent_role": "researcher",
        "consciousness_mode": "full"
      }
    }
  }
}
```

## Supported Exa Tools

All Exa MCP tools are supported through the `orchestrate_mcp_proxy` tool:

- **exa.search** - Semantic and keyword search with domain filtering
- **exa.get_contents** - Content retrieval with subpages and livecrawling
- **exa.find_similar** - Similarity search based on content
- **exa.answer** - Direct question answering with citations
- **exa.websets.create/search** - Standing queries with webhooks
- **exa.research.create_task** - Multi-step research orchestration

## Key Features

### 1. Automatic Context Injection

Every Exa tool call receives:
- Current date/time for temporal awareness
- Tool-specific guidance from templates
- Agent role-based instructions
- Pitfall warnings to prevent common errors

### 2. Multi-Agent Coordination

The Exa mission implements a five-agent pattern:

```
Planner → Websets → Crawlers → Synthesizer → Verifier
```

### 3. Domain Filtering

```json
{
  "domain_filters": {
    "include_domains": ["*.edu", "*/research/*", "*.github.com"],
    "exclude_domains": ["*.medium.com", "*.substack.com"]
  }
}
```

### 4. Livecrawling Control

- **never** - Use cached content only
- **fallback** - Try cache first, crawl if stale
- **preferred** - Fresh crawl for time-sensitive content (default for research)
- **always** - Force fresh crawl

## Example: Research Task

```python
# Step 1: Discover available Exa tools
discover_response = await mcp_call("discover_mcp_tools", {
    "server_url": "https://exa-mcp.api",
    "analyze_for_orchestration": True
})

# Step 2: Create a research task with full augmentation
research_response = await mcp_call("orchestrate_mcp_proxy", {
    "target_server": "https://exa-mcp.api",
    "tool_name": "exa.research.create_task",
    "original_params": {
        "instruction": "Analyze quantum computing breakthroughs in 2024",
        "outputSchema": {
            "type": "object",
            "properties": {
                "breakthroughs": {"type": "array"},
                "key_players": {"type": "array"},
                "implications": {"type": "object"}
            }
        }
    },
    "augmentation_config": {
        "inject_context": True,
        "perception_ids": ["exa_research_consciousness"],
        "add_swarm_instructions": [
            "Focus on peer-reviewed sources",
            "Include industry applications",
            "Track funding patterns"
        ],
        "domain_filters": {
            "include_domains": ["*.arxiv.org", "*.nature.com", "*.science.org"]
        },
        "livecrawl_mode": "preferred",
        "paradox_tolerance": 0.7
    }
})
```

## Templates and Perceptions

### Available Templates

1. **exa_framework** - Global Exa orchestration principles
2. **exa_search** - Search optimization and query expansion
3. **exa_crawling** - Content retrieval strategies
4. **exa_websets** - Standing query configuration
5. **exa_research** - Multi-step research planning
6. **exa_citations** - Citation standards and verification

### Consciousness Perceptions

1. **exa_research_consciousness** - Applied to research and answer tools
2. **exa_monitoring_consciousness** - Applied to websets
3. **exa_deep_crawl_consciousness** - Applied to content retrieval

## Agent Roles

Set via `agent_role` parameter or `x-agent-role` header:

- **researcher** - Full research consciousness with citation requirements
- **analyst** - Analytical focus with evidence chains
- **monitor/watcher** - Optimized for websets and alerts
- **orchestrator** - Multi-agent coordination capabilities

## Advanced Configuration

### Custom Rule Mapping

Add to your mission file:

```yaml
rules:
  - id: custom_exa_rule
    condition: |
      tool_name == 'exa.search' AND 
      query contains 'breaking news'
    action: apply_perception
    perception_id: exa_monitoring_consciousness
    overrides:
      livecrawl_mode: "always"
```

### Session-Level Configuration

```bash
curl "http://localhost:8080/mcp?apiKey=${MOP_API_KEY:-DEMO_KEY_PUBLIC}\
&agent_role=researcher\
&consciousness_mode=full\
&max_context_size=2000"
```

## Monitoring and Debugging

### View Current Shim Configuration

```bash
curl http://localhost:8080/debug/shim
```

### Check Applied Templates

Enable debug mode to see which templates are being applied:

```bash
curl "http://localhost:8080/mcp?apiKey=${MOP_API_KEY:-DEMO_KEY_PUBLIC}&debug=true"
```

### Audit Log

All augmentations are logged with:
- Template versions applied
- Injection reasoning
- Token budget usage
- Performance metrics

## Best Practices

1. **Start with the Mission** - Always load `exa-mcp-orchestration.yaml`
2. **Set Agent Roles** - Explicitly set roles for appropriate context
3. **Use Domain Filters** - Focus searches on authoritative sources
4. **Configure Livecrawl** - Use "preferred" for research, "fallback" for monitoring
5. **Monitor Token Usage** - Templates are budgeted to ~1200 tokens
6. **Enable Citations** - Research tools should always maintain citation chains

## Integration Examples

### With Claude Desktop

```json
{
  "mcpServers": {
    "casial-exa": {
      "command": "node",
      "args": ["path/to/casial-mcp-client.js"],
      "env": {
        "CASIAL_URL": "http://localhost:8080/mcp",
        "CASIAL_API_KEY": "${MOP_API_KEY:-DEMO_KEY_PUBLIC}",
        "AGENT_ROLE": "researcher"
      }
    }
  }
}
```

### With Python Client

```python
import asyncio
from mcp import Client

async def main():
    client = Client(
        "http://localhost:8080/mcp",
        config={
            "apiKey": "${MOP_API_KEY:-DEMO_KEY_PUBLIC}",
            "agent_role": "researcher"
        }
    )
    
    # Use orchestrated Exa search
    result = await client.call_tool("orchestrate_mcp_proxy", {
        "target_server": "https://exa-mcp.api",
        "tool_name": "exa.search",
        "original_params": {"query": "AI safety research"},
        "augmentation_config": {
            "inject_context": True,
            "perception_ids": ["exa_research_consciousness"]
        }
    })
```

## Troubleshooting

### Templates Not Applied

1. Check mission is loaded: `curl http://localhost:8080/debug/missions`
2. Verify perception mappings match tool names
3. Ensure agent_role is set correctly

### Token Budget Exceeded

1. Reduce number of templates in perception
2. Use more specific perceptions (not all templates)
3. Adjust `max_context_size` in session config

### Livecrawl Timeouts

1. Start with "fallback" mode
2. Only use "always" for critical fresh content
3. Consider implementing retry logic

## Next Steps

1. Explore other turnkey integrations (coming soon)
2. Create custom missions for your use case
3. Contribute templates back to the community
4. Join our Discord for support