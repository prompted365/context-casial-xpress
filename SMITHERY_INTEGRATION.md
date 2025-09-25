# Smithery Integration Guide for MOP

## Key Changes Made for Smithery Compatibility

### 1. Port Configuration
- **Changed default port from 8080 to 8081**
- Smithery expects servers to bind to PORT environment variable (defaults to 8081)
- Updated in both `railway-start.sh` and `smithery.yaml`

### 2. Well-Known Endpoint Format
- Restructured `/.well-known/mcp-config` response
- Moved JSON Schema fields into nested `configSchema` property
- Top-level now only contains metadata (name, version, vendor, etc.)
- Added POST support for potential JSON-RPC requests

### 3. CORS Headers
- **Allowed headers**: `Authorization`, `Mop-Admin-Token`, `Content-Type`, and MCP session headers
- Smithery caches should respect `Vary: Origin, Authorization, Mop-Admin-Token` to avoid credential mix-ups.
- **Exposed headers**: `mcp-session-id`, `mcp-protocol-version`
- **Credentials**: Enabled when `ALLOWED_ORIGINS` is a comma-delimited allow list; disabled automatically for `*`
- **Methods**: GET, POST, OPTIONS

### 4. Server Metadata
Added required fields to indicate deployed server:
- `vendor`: "Prompted LLC"
- `homepage`: GitHub repository URL
- `transport`: ["streamable-http"]
- `capabilities`: Tools enabled, others disabled

## Troubleshooting Smithery Issues

### "Server is local" Error
This means Smithery thinks your server runs locally (stdio transport) instead of being deployed.
**Solution**: Ensure your well-known endpoint returns proper metadata with vendor and homepage.

### "401 on POST" Error  
Smithery might POST to well-known endpoint during scanning.
**Solution**: We added POST handler that can forward JSON-RPC or return config.

### Capabilities Not Listed
Ensure:
1. Server binds to correct PORT (8081)
2. CORS headers are properly configured
3. `/mcp` endpoint responds to `initialize` and `tools/list` methods
4. Authentication is handled correctly (Authorization Bearer header)

## Testing Your Server

```bash
# Test well-known endpoint
curl https://your-server.railway.app/.well-known/mcp-config

# Test MCP initialize
curl -X POST "https://your-server.railway.app/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${MOP_API_KEY:-DEMO_KEY_PUBLIC}" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}},"id":1}'

# Test tools list
curl -X POST "https://your-server.railway.app/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${MOP_API_KEY:-DEMO_KEY_PUBLIC}" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
```

## Expected Smithery Flow

1. Smithery scans your GitHub repository
2. Finds `smithery.yaml` with container runtime
3. Builds and runs your Docker container
4. Makes requests to `/.well-known/mcp-config` (GET and possibly POST)
5. Connects to `/mcp` endpoint with user's configuration
6. Calls `initialize` to establish session
7. Calls `tools/list` to discover capabilities
8. Routes tool calls from users to your server

## Current Configuration

- **Server Name**: meta-orchestration-protocol
- **Default Port**: 8081 (via PORT env var)
- **Transport**: streamable-http
- **API Key**: Send `Authorization: Bearer ${MOP_API_KEY:-DEMO_KEY_PUBLIC}` _(DEMO KEY – public)_
- **Endpoints**:
  - `/health` - Health check
  - `/mcp` - Main MCP endpoint (HTTP/SSE)
  - `/.well-known/mcp-config` - Configuration discovery
  - `/ws` - WebSocket endpoint (alternative transport)