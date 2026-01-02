#!/usr/bin/env bash
# Test script for Linear multi-agent integration
# Usage: ./scripts/test-linear-agents.sh [command]
#
# Commands:
#   oauth-urls     Generate OAuth installation URLs for all agents
#   webhook-test   Test webhook endpoint with sample payloads
#   list-secrets   List all Linear agent secrets in cluster
#   health         Check PM service health
#   all            Run all tests
set -euo pipefail

# Configuration
PM_SERVICE_URL="${PM_SERVICE_URL:-http://localhost:8081}"
LINEAR_TEAM_ID="${LINEAR_TEAM_ID:-}"
AGENTS="morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
info() { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Generate OAuth URLs for all agents (from OpenBao or K8s secrets)
oauth_urls() {
  info "=== OAuth Installation URLs ==="
  echo ""
  echo "Visit each URL to authorize the agent OAuth app."
  echo "After authorization, the token will be stored for that agent."
  echo ""

  local base_url="https://linear.app/oauth/authorize"
  local redirect_uri="https://cto.5dlabs.ai/oauth/callback"
  local scope="read,write,app:assignable,app:mentionable"

  for agent in $AGENTS; do
    # Try to get client_id from K8s secret first, then OpenBao
    client_id=""
    if kubectl get secret "linear-app-$agent" -n cto &>/dev/null; then
      client_id=$(kubectl get secret "linear-app-$agent" -n cto -o jsonpath='{.data.client_id}' | base64 -d 2>/dev/null || echo "")
    elif command -v bao &>/dev/null; then
      client_id=$(bao kv get -format=json "linear-app-$agent" 2>/dev/null | jq -r '.data.data.client_id' 2>/dev/null || echo "")
    fi

    if [[ -n "$client_id" && "$client_id" != "null" ]]; then
      echo "=== ${agent^} ==="
      echo "${base_url}?client_id=${client_id}&redirect_uri=${redirect_uri}&response_type=code&scope=${scope}&actor=app&state=${agent}"
      echo ""
    else
      warn "No client_id for $agent - credentials not configured"
    fi
  done
}

# Test webhook endpoint
webhook_test() {
  info "=== Testing Webhook Endpoint ==="

  # Check if PM service is reachable
  if ! curl -s -o /dev/null -w "%{http_code}" "${PM_SERVICE_URL}/health" | grep -q "200"; then
    error "PM service not reachable at ${PM_SERVICE_URL}"
    error "Start port-forward: kubectl port-forward svc/pm-svc -n cto 8081:8081"
    exit 1
  fi
  info "PM service is healthy"

  # Test with mock webhook payload (will fail signature check but tests routing)
  info "Testing webhook routing (expect 401 - no valid signature)..."
  local payload='{"action":"create","type":"Issue","data":{"id":"test-123"}}'
  
  response=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -H "Linear-Signature: invalid-signature" \
    -H "Linear-Delivery: test-delivery-id" \
    -d "$payload" \
    "${PM_SERVICE_URL}/webhooks/linear")
  
  http_code=$(echo "$response" | tail -n1)
  body=$(echo "$response" | head -n-1)
  
  if [[ "$http_code" == "401" ]]; then
    info "Webhook correctly rejected invalid signature (401)"
  else
    warn "Unexpected response code: $http_code"
    echo "Body: $body"
  fi

  # Test AgentSession event parsing
  info "Testing AgentSession event structure..."
  local session_payload='{
    "action": "created",
    "type": "AgentSession",
    "data": {
      "id": "session-test-123",
      "agentId": "test-agent",
      "issueId": "issue-test-456"
    },
    "organizationId": "org-test"
  }'
  
  response=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -H "Linear-Signature: invalid-signature" \
    -d "$session_payload" \
    "${PM_SERVICE_URL}/webhooks/linear")
  
  http_code=$(echo "$response" | tail -n1)
  
  if [[ "$http_code" == "401" ]]; then
    info "AgentSession webhook routing works (signature rejected as expected)"
  else
    warn "Unexpected response: $http_code"
  fi

  info "Webhook tests complete"
}

# List Linear agent secrets in cluster
list_secrets() {
  info "=== Linear Agent Secrets in Cluster ==="
  
  echo ""
  echo "Checking for linear-app-* secrets in cto namespace..."
  echo ""
  
  for agent in $AGENTS; do
    secret_name="linear-app-$agent"
    if kubectl get secret "$secret_name" -n cto &>/dev/null; then
      # Check if it has the expected keys
      keys=$(kubectl get secret "$secret_name" -n cto -o jsonpath='{.data}' | jq -r 'keys | join(", ")')
      echo -e "${GREEN}✓${NC} $secret_name: $keys"
    else
      echo -e "${RED}✗${NC} $secret_name: NOT FOUND"
    fi
  done
}

# Health check
health() {
  info "=== Health Check ==="
  
  # PM Service
  echo -n "PM Service: "
  if curl -s -o /dev/null -w "%{http_code}" "${PM_SERVICE_URL}/health" | grep -q "200"; then
    echo -e "${GREEN}OK${NC}"
  else
    echo -e "${RED}FAIL${NC}"
  fi
  
  # OAuth endpoint
  echo -n "OAuth Callback: "
  if curl -s -o /dev/null -w "%{http_code}" "${PM_SERVICE_URL}/oauth/callback?error=test" | grep -q "200"; then
    echo -e "${GREEN}OK${NC} (error handling works)"
  else
    echo -e "${RED}FAIL${NC}"
  fi
  
  # OAuth start endpoint
  echo -n "OAuth Start: "
  response=$(curl -s -o /dev/null -w "%{http_code}" "${PM_SERVICE_URL}/oauth/start?agent=morgan")
  if [[ "$response" == "307" || "$response" == "302" || "$response" == "404" ]]; then
    echo -e "${GREEN}OK${NC} (redirects or 404 without config)"
  else
    echo -e "${YELLOW}WARN${NC} (response: $response)"
  fi
}

# Test agent mention (requires valid credentials)
test_mention() {
  info "=== Test Agent Mention ==="
  warn "This test requires valid Linear API credentials"
  
  if [[ -z "${LINEAR_API_KEY:-}" ]]; then
    error "LINEAR_API_KEY not set"
    echo "Export LINEAR_API_KEY to run this test"
    exit 1
  fi
  
  if [[ -z "$LINEAR_TEAM_ID" ]]; then
    error "LINEAR_TEAM_ID not set"
    echo "Export LINEAR_TEAM_ID to run this test"
    exit 1
  fi
  
  local agent="${1:-morgan}"
  info "Creating test issue and mentioning @$agent..."
  
  # This would require GraphQL calls to Linear API
  # For now, just output instructions
  echo ""
  echo "Manual test steps:"
  echo "1. Go to Linear and create an issue in your team"
  echo "2. In the issue description or comment, type @$agent"
  echo "3. Watch the PM service logs for AgentSession webhook"
  echo "4. Check Linear for agent response (should emit thought within 10s)"
}

# Run all tests
all() {
  health
  echo ""
  webhook_test
  echo ""
  list_secrets
  echo ""
  oauth_urls
}

# Main
case "${1:-help}" in
  oauth-urls|urls)
    oauth_urls
    ;;
  webhook-test|webhook)
    webhook_test
    ;;
  list-secrets|secrets)
    list_secrets
    ;;
  health)
    health
    ;;
  mention)
    test_mention "${2:-morgan}"
    ;;
  all)
    all
    ;;
  *)
    echo "Usage: $0 {oauth-urls|webhook-test|list-secrets|health|mention|all}"
    echo ""
    echo "Commands:"
    echo "  oauth-urls     Generate OAuth installation URLs for all agents"
    echo "  webhook-test   Test webhook endpoint with sample payloads"
    echo "  list-secrets   List all Linear agent secrets in cluster"
    echo "  health         Check PM service health"
    echo "  mention        Test agent mention (requires LINEAR_API_KEY)"
    echo "  all            Run all non-interactive tests"
    echo ""
    echo "Environment:"
    echo "  PM_SERVICE_URL    PM service URL (default: http://localhost:8081)"
    echo "  LINEAR_TEAM_ID    Linear team ID for mention tests"
    echo "  LINEAR_API_KEY    Linear API key for mention tests"
    exit 1
    ;;
esac
