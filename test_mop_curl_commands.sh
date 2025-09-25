#!/bin/bash

# Meta-Orchestration Protocol (MOP) cURL Test Commands

API_KEY="${MOP_API_KEY:-DEMO_KEY_PUBLIC}"

echo "=== 1. Initialize Session ==="
SESSION_ID=$(curl -X POST https://context-casial-xpress-production.up.railway.app/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "curl-test",
        "version": "1.0.0"
      }
    },
    "id": 1
  }' -s | jq -r '.result.sessionId')

echo "Session ID: $SESSION_ID"

echo -e "\n=== 2. List Tools ==="
curl -X POST https://context-casial-xpress-production.up.railway.app/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 2
  }' -s | jq '.result.tools[].name'

echo -e "\n=== 3. List Prompts ==="
curl -X POST https://context-casial-xpress-production.up.railway.app/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "prompts/list",
    "params": {},
    "id": 3
  }' -s | jq '.result.prompts[].name'

echo -e "\n=== 4. List Resources ==="
curl -X POST https://context-casial-xpress-production.up.railway.app/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "resources/list",
    "params": {},
    "id": 4
  }' -s | jq '.result.resources[].name'

echo -e "\n=== 5. Call Orchestrate Tool (Example) ==="
curl -X POST https://context-casial-xpress-production.up.railway.app/mcp \
  -H "Content-Type: application/json" \
  -H "mcp-session-id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "discover_mcp_tools",
      "arguments": {
        "server_url": "https://github.com/modelcontextprotocol/servers",
        "analyze_for_orchestration": true,
        "perception_mapping": false
      }
    },
    "id": 5
  }' -s | jq '.'

echo -e "\n=== 6. Terminate Session ==="
curl -X DELETE https://context-casial-xpress-production.up.railway.app/mcp \
  -H "mcp-session-id: $SESSION_ID" \
  -i 2>&1 | grep "HTTP/1.1"