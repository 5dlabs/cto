#!/bin/bash

# Test script for doc-server ingress configuration
# This script tests the SSE-optimized NGINX ingress setup

set -e

# Configuration
BASE_URL="http://mcp.simple-cluster.local"
NGINX_NODEPORT="31251"  # From Talos README
CONTROL_PLANE_IP="192.168.1.77"  # From Talos config

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing doc-server ingress configuration...${NC}"
echo "Base URL: $BASE_URL"
echo "NGINX NodePort: $NGINX_NODEPORT"
echo "Control Plane IP: $CONTROL_PLANE_IP"
echo ""

# Test 1: Preflight request (CORS)
echo -e "${YELLOW}1. Testing CORS preflight request...${NC}"
if curl -i -X OPTIONS "$BASE_URL/mcp" \
    -H 'Origin: http://internal' \
    -H 'Access-Control-Request-Method: POST' \
    --connect-timeout 10 \
    --max-time 30 \
    --fail-with-body 2>/dev/null | grep -q "204\|200"; then
    echo -e "${GREEN}✓ CORS preflight successful${NC}"
else
    echo -e "${RED}✗ CORS preflight failed${NC}"
    echo "Trying with NodePort directly..."
    if curl -i -X OPTIONS "http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/mcp" \
        -H 'Host: mcp.simple-cluster.local' \
        -H 'Origin: http://internal' \
        -H 'Access-Control-Request-Method: POST' \
        --connect-timeout 10 \
        --max-time 30 \
        --fail-with-body 2>/dev/null | grep -q "204\|200"; then
        echo -e "${GREEN}✓ CORS preflight successful via NodePort${NC}"
    else
        echo -e "${RED}✗ CORS preflight failed via NodePort${NC}"
    fi
fi

echo ""

# Test 2: Initialize request
echo -e "${YELLOW}2. Testing MCP initialize request...${NC}"
if curl -i -sS -X POST "$BASE_URL/mcp" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}' \
    --connect-timeout 10 \
    --max-time 30 \
    --fail-with-body 2>/dev/null | grep -q "jsonrpc"; then
    echo -e "${GREEN}✓ MCP initialize successful${NC}"
else
    echo -e "${RED}✗ MCP initialize failed${NC}"
    echo "Trying with NodePort directly..."
    if curl -i -sS -X POST "http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/mcp" \
        -H 'Host: mcp.simple-cluster.local' \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}' \
        --connect-timeout 10 \
        --max-time 30 \
        --fail-with-body 2>/dev/null | grep -q "jsonrpc"; then
        echo -e "${GREEN}✓ MCP initialize successful via NodePort${NC}"
    else
        echo -e "${RED}✗ MCP initialize failed via NodePort${NC}"
    fi
fi

echo ""

# Test 3: Tools list request
echo -e "${YELLOW}3. Testing MCP tools/list request...${NC}"
if curl -i -sS -X POST "$BASE_URL/mcp" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
    --connect-timeout 10 \
    --max-time 30 \
    --fail-with-body 2>/dev/null | grep -q "jsonrpc"; then
    echo -e "${GREEN}✓ MCP tools/list successful${NC}"
else
    echo -e "${RED}✗ MCP tools/list failed${NC}"
    echo "Trying with NodePort directly..."
    if curl -i -sS -X POST "http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/mcp" \
        -H 'Host: mcp.simple-cluster.local' \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        --connect-timeout 10 \
        --max-time 30 \
        --fail-with-body 2>/dev/null | grep -q "jsonrpc"; then
        echo -e "${GREEN}✓ MCP tools/list successful via NodePort${NC}"
    else
        echo -e "${RED}✗ MCP tools/list failed via NodePort${NC}"
    fi
fi

echo ""

# Test 4: SSE connection (Server-Sent Events)
echo -e "${YELLOW}4. Testing SSE connection...${NC}"
echo "This will test the SSE endpoint for 10 seconds..."
if timeout 10 curl -N -H 'Accept: text/event-stream' "$BASE_URL/mcp" \
    --connect-timeout 10 \
    --fail-with-body 2>/dev/null | grep -q "data:"; then
    echo -e "${GREEN}✓ SSE connection successful${NC}"
else
    echo -e "${RED}✗ SSE connection failed${NC}"
    echo "Trying with NodePort directly..."
    if timeout 10 curl -N -H 'Accept: text/event-stream' "http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/mcp" \
        -H 'Host: mcp.simple-cluster.local' \
        --connect-timeout 10 \
        --fail-with-body 2>/dev/null | grep -q "data:"; then
        echo -e "${GREEN}✓ SSE connection successful via NodePort${NC}"
    else
        echo -e "${RED}✗ SSE connection failed via NodePort${NC}"
    fi
fi

echo ""

# Test 5: Health check
echo -e "${YELLOW}5. Testing health endpoint...${NC}"
if curl -i -sS "$BASE_URL/health" \
    --connect-timeout 10 \
    --max-time 30 \
    --fail-with-body 2>/dev/null | grep -q "200\|OK"; then
    echo -e "${GREEN}✓ Health check successful${NC}"
else
    echo -e "${RED}✗ Health check failed${NC}"
    echo "Trying with NodePort directly..."
    if curl -i -sS "http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/health" \
        -H 'Host: mcp.simple-cluster.local' \
        --connect-timeout 10 \
        --max-time 30 \
        --fail-with-body 2>/dev/null | grep -q "200\|OK"; then
        echo -e "${GREEN}✓ Health check successful via NodePort${NC}"
    else
        echo -e "${RED}✗ Health check failed via NodePort${NC}"
    fi
fi

echo ""
echo -e "${YELLOW}Ingress testing complete!${NC}"
echo ""
echo "If tests failed, check:"
echo "1. ArgoCD has synced the doc-server application"
echo "2. NGINX ingress controller is running"
echo "3. DNS resolution for mcp.simple-cluster.local"
echo "4. Firewall rules allow access to NodePort 31251"
echo ""
echo "To access the service directly via NodePort:"
echo "curl -H 'Host: mcp.simple-cluster.local' http://$CONTROL_PLANE_IP:$NGINX_NODEPORT/mcp"
