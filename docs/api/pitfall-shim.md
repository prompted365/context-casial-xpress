# Global Pitfall Avoidance Shim API

The Global Pitfall Avoidance Shim is a quality-of-life feature that automatically augments all MCP tool calls with helpful context to prevent common AI pitfalls.

## Overview

The shim operates as a transparent layer that:
- Injects current date/time information by default
- Adds timestamps to all tool responses
- Provides contextual warnings for specific tools
- Can be configured and extended via command-line or API

## Command-Line Options

```bash
# Enable shim (default)
casial-server start

# Disable shim
casial-server start --no-shim

# Add custom extension
casial-server start --shim-extend "Project: MyAI, Environment: Production"

# Use custom shim configuration
casial-server start --shim-config path/to/shim-config.json
```

## Configuration Structure

```json
{
  "enabled": true,
  "inject_datetime": true,
  "timestamp_returns": true,
  "custom_extension": "Optional custom string",
  "features": {
    "inject_timezone": true,
    "add_execution_metadata": true,
    "include_system_info": false,
    "date_format_hints": true,
    "pitfall_warnings": true
  }
}
```

## Injected Context

When enabled, the shim adds a `_shim_context` object to all tool requests:

```json
{
  "_shim_context": {
    "current_datetime_utc": "2024-01-15T10:30:00Z",
    "current_datetime_local": "2024-01-15T02:30:00-08:00",
    "current_date": "2024-01-15",
    "current_time": "02:30:00",
    "timezone": "PST",
    "timezone_offset": "-0800",
    "date_format_hints": {
      "iso8601": "2024-01-15T10:30:00Z",
      "unix_timestamp": 1705318200,
      "human_readable": "January 15, 2024 at 02:30 AM PST",
      "sortable": "20240115_023000"
    },
    "execution_metadata": {
      "tool_name": "web_search_exa",
      "shim_version": "1.0.0",
      "request_id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": 1705318200000
    },
    "pitfall_warnings": [
      "Current date is 2024-01-15 - ensure any date-based queries use this as reference",
      "When searching for recent events, remember to include the current year in queries"
    ],
    "custom_extension": "Project: MyAI, Environment: Production"
  }
}
```

## Response Metadata

All tool responses are augmented with `_response_metadata`:

```json
{
  "_response_metadata": {
    "processed_at": "2024-01-15T10:30:00Z",
    "processing_time_ms": 42,
    "tool_name": "web_search_exa",
    "shim_applied": true
  }
}
```

## REST API Endpoints

### View Shim Configuration

```bash
GET /debug/shim

Response:
{
  "shim_status": {
    "enabled": true,
    "inject_datetime": true,
    "timestamp_returns": true,
    "custom_extension": null,
    "features": {...}
  },
  "current_context_example": {
    "current_date": "2024-01-15",
    "current_time": "02:30:00",
    "timezone": "PST"
  },
  "edit_instructions": "POST to /debug/shim with JSON configuration to update"
}
```

### Update Shim Configuration

```bash
POST /debug/shim
Content-Type: application/json

{
  "enabled": true,
  "inject_datetime": true,
  "timestamp_returns": false,
  "custom_extension": "New custom context",
  "features": {
    "inject_timezone": true,
    "add_execution_metadata": false,
    "include_system_info": true,
    "date_format_hints": true,
    "pitfall_warnings": true
  }
}

Response:
{
  "status": "success",
  "message": "Shim configuration updated",
  "new_config": {...}
}
```

## Tool-Specific Warnings

The shim provides contextual warnings based on the tool being called:

### Search Tools (web_search_exa, deep_researcher_start)
- "When searching for recent events, remember to include the current year in queries"
- "For documentation searches, prefer recent versions by including year constraints"

### Orchestration Tools (orchestrate_mcp_proxy)
- "Ensure target server URLs are properly validated before proxying"
- "Consider consciousness coordination impacts on downstream servers"

## Custom Shim Configuration File

Create a JSON file with your preferred configuration:

```json
{
  "enabled": true,
  "inject_datetime": true,
  "timestamp_returns": true,
  "custom_extension": "Production Environment - Handle with Care",
  "features": {
    "inject_timezone": true,
    "add_execution_metadata": true,
    "include_system_info": true,
    "date_format_hints": true,
    "pitfall_warnings": true
  }
}
```

Then start the server with:
```bash
casial-server start --shim-config my-shim-config.json
```

## Best Practices

1. **Keep shim enabled in production** - Prevents common date/time errors
2. **Use custom extensions** - Add project/environment context
3. **Monitor warnings** - Review pitfall warnings in logs
4. **Test with shim disabled** - Ensure tools work without augmentation
5. **Configure per environment** - Different settings for dev/staging/prod

## Integration with MCP Orchestration

When using the `orchestrate_mcp_proxy` tool, the shim context is automatically forwarded to downstream MCP servers, ensuring consistent context across your entire tool ecosystem.

Example flow:
1. Client calls `orchestrate_mcp_proxy` 
2. Shim adds current date/time and warnings
3. Proxy forwards augmented request to target MCP server
4. Response includes both proxy and shim metadata

This creates a consciousness-aware layer across all your MCP integrations.