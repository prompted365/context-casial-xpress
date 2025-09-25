#!/bin/bash
# Test script for Smithery MCP compatibility

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing Meta-Orchestration Protocol (MOP) Smithery Compatibility${NC}"
echo "=================================================="

# Base URL (adjust if testing on production)
BASE_URL="${MCP_TEST_URL:-http://localhost:8080}"
API_KEY="${MOP_API_KEY:-DEMO_KEY_PUBLIC}"

# Test 1: Health Check
echo -e "\n${YELLOW}Test 1: Health Check${NC}"
response=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/health")
if [ "$response" = "200" ]; then
    echo -e "${GREEN}✓ Health check passed${NC}"
else
    echo -e "${RED}✗ Health check failed (HTTP $response)${NC}"
fi

# Test 2: Well-known MCP Config
echo -e "\n${YELLOW}Test 2: Well-known MCP Config${NC}"
config_response=$(curl -s "${BASE_URL}/.well-known/mcp-config")
if echo "$config_response" | jq -e '.configSchema.properties.apiKey' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ MCP config endpoint working${NC}"
    echo "  Transport: $(echo "$config_response" | jq -r '.transport[]')"
    echo "  Name: $(echo "$config_response" | jq -r '.name')"
else
    echo -e "${RED}✗ MCP config endpoint failed${NC}"
fi

# Test 3: CORS Preflight
echo -e "\n${YELLOW}Test 3: CORS Preflight${NC}"
cors_response=$(curl -s -I -X OPTIONS "${BASE_URL}/mcp" \
    -H "Origin: https://smithery.ai" \
    -H "Access-Control-Request-Method: POST" \
    -H "Access-Control-Request-Headers: Content-Type" | grep -i "access-control")
if [ -n "$cors_response" ]; then
    echo -e "${GREEN}✓ CORS headers present${NC}"
    echo "$cors_response"
else
    echo -e "${RED}✗ CORS headers missing${NC}"
fi

# Test 4: MCP Initialize (with auth)
echo -e "\n${YELLOW}Test 4: MCP Initialize${NC}"
init_response=$(curl -s -X POST "${BASE_URL}/mcp" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${API_KEY}" \
    -d '{
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "smithery-test",
                "version": "1.0.0"
            }
        },
        "id": 1
    }')

if echo "$init_response" | jq -e '.result.serverInfo' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ MCP initialize succeeded${NC}"
    echo "  Server: $(echo "$init_response" | jq -r '.result.serverInfo.name')"
    echo "  Version: $(echo "$init_response" | jq -r '.result.serverInfo.version')"
else
    echo -e "${RED}✗ MCP initialize failed${NC}"
    echo "$init_response" | jq '.'
fi

# Test 5: Base64 Config Parameter
echo -e "\n${YELLOW}Test 5: Base64 Config Parameter${NC}"
config_json=$(jq -n --arg key "$API_KEY" '{"apiKey":$key,"agent_role":"researcher","consciousness_mode":"full"}')
encoded_config=$(echo -n "$config_json" | base64)
base64_response=$(curl -s -X POST "${BASE_URL}/mcp?config=${encoded_config}" \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 2
    }')

if echo "$base64_response" | jq -e '.result.tools' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Base64 config parameter working${NC}"
    echo "  Tools count: $(echo "$base64_response" | jq '.result.tools | length')"
else
    echo -e "${RED}✗ Base64 config parameter failed${NC}"
    echo "$base64_response" | jq '.'
fi

# Test 6: SSE Stream Support
echo -e "\n${YELLOW}Test 6: SSE Stream Support${NC}"
sse_headers=$(curl -s -I "${BASE_URL}/mcp" \
    -H "Accept: text/event-stream" \
    -H "Authorization: Bearer ${API_KEY}" | grep -i "content-type")
if echo "$sse_headers" | grep -q "text/event-stream"; then
    echo -e "${GREEN}✓ SSE stream support confirmed${NC}"
else
    echo -e "${RED}✗ SSE stream not supported${NC}"
fi

# Test 7: Authentication Failure
echo -e "\n${YELLOW}Test 7: Authentication Failure Test${NC}"
auth_fail_response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${BASE_URL}/mcp" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer wrong-key" \
    -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}')
if [ "$auth_fail_response" = "401" ]; then
    echo -e "${GREEN}✓ Authentication properly enforced${NC}"
else
    echo -e "${RED}✗ Authentication not enforced (HTTP $auth_fail_response)${NC}"
fi

echo -e "\n${YELLOW}Summary${NC}"
echo "=================================================="
echo -e "${GREEN}Smithery compatibility tests complete!${NC}"
echo ""
echo "To register with Smithery:"
echo "1. Ensure this repository is public on GitHub"
echo "2. Push the smithery.yaml file to the root directory"
echo "3. Visit https://smithery.ai and add your repository"
echo "4. Smithery will scan for the smithery.yaml file"
echo "5. Your MCP server will be available at:"
echo "   https://your-deployment.smithery.ai/mcp (send Authorization: Bearer ${API_KEY})"