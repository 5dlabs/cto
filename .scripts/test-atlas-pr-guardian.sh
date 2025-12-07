#!/bin/bash
set -euo pipefail

# Test Atlas PR Guardian Sensor Configuration
# Validates sensor YAML, checks deployment, and simulates webhook events

echo "╔════════════════════════════════════════════════════════════╗"
echo "║   Atlas PR Guardian - Configuration Validation            ║"
echo "╔════════════════════════════════════════════════════════════╗"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
  echo -e "${GREEN}✅ PASS${NC}: $1"
  ((TESTS_PASSED++))
}

fail() {
  echo -e "${RED}❌ FAIL${NC}: $1"
  ((TESTS_FAILED++))
}

warn() {
  echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

info() {
  echo -e "ℹ️  $1"
}

# Test 1: Validate sensor YAML syntax
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Validate Sensor YAML Syntax"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SENSOR_FILE="$PROJECT_ROOT/infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml"

if [ ! -f "$SENSOR_FILE" ]; then
  fail "Sensor file not found: $SENSOR_FILE"
else
  if yamllint "$SENSOR_FILE" 2>/dev/null; then
    pass "Sensor YAML syntax is valid"
  else
    warn "yamllint not installed or YAML has warnings (non-critical)"
    # Try basic YAML parsing
    if python3 -c "import yaml; yaml.safe_load(open('$SENSOR_FILE'))" 2>/dev/null; then
      pass "Sensor YAML is parseable"
    else
      fail "Sensor YAML syntax is invalid"
    fi
  fi
fi

# Test 2: Validate ArgoCD application YAML
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Validate ArgoCD Application YAML"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

APP_FILE="$PROJECT_ROOT/infra/gitops/applications/atlas-pr-guardian-sensor.yaml"

if [ ! -f "$APP_FILE" ]; then
  fail "ArgoCD application file not found: $APP_FILE"
else
  if yamllint "$APP_FILE" 2>/dev/null; then
    pass "ArgoCD application YAML syntax is valid"
  else
    warn "yamllint not installed or YAML has warnings (non-critical)"
    if python3 -c "import yaml; yaml.safe_load(open('$APP_FILE'))" 2>/dev/null; then
      pass "ArgoCD application YAML is parseable"
    else
      fail "ArgoCD application YAML syntax is invalid"
    fi
  fi
fi

# Test 3: Check sensor structure
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: Validate Sensor Structure"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for required fields
if grep -q "kind: Sensor" "$SENSOR_FILE"; then
  pass "Sensor has correct kind"
else
  fail "Sensor missing 'kind: Sensor'"
fi

if grep -q "name: atlas-pr-guardian" "$SENSOR_FILE"; then
  pass "Sensor has correct name"
else
  fail "Sensor missing name 'atlas-pr-guardian'"
fi

if grep -q "dependencies:" "$SENSOR_FILE"; then
  pass "Sensor has dependencies section"
else
  fail "Sensor missing dependencies"
fi

if grep -q "triggers:" "$SENSOR_FILE"; then
  pass "Sensor has triggers section"
else
  fail "Sensor missing triggers"
fi

# Test 4: Check event dependencies
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Validate Event Dependencies"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Note: Sensor uses single dependency due to Argo Events v1.9.7 constraint
# Argo Events doesn't support multiple dependencies with same eventName
# pr-events: Handles both pull_request and issue_comment events

if grep -q "name: pr-events" "$SENSOR_FILE"; then
  pass "Single pr-events dependency configured (Argo Events v1.9.7 compliant)"
else
  fail "pr-events dependency missing"
fi

# Check that we don't have multiple dependencies (causes InvalidDependencies error)
if grep -q "name: pr-lifecycle" "$SENSOR_FILE"; then
  warn "Multiple dependencies found (Argo Events v1.9.7 doesn't support this)"
fi

if grep -q "name: pr-comments" "$SENSOR_FILE"; then
  warn "Multiple dependencies found (Argo Events v1.9.7 doesn't support this)"
fi

# Test 5: Check CodeRun trigger
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 5: Validate CodeRun Trigger"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if grep -q "kind: CodeRun" "$SENSOR_FILE"; then
  pass "CodeRun resource type configured"
else
  fail "CodeRun resource type missing"
fi

if grep -q "githubApp: \"5DLabs-Atlas\"" "$SENSOR_FILE"; then
  pass "GitHub App set to 5DLabs-Atlas"
else
  fail "GitHub App not set correctly"
fi

if grep -q "continueSession: true" "$SENSOR_FILE"; then
  pass "Session continuity enabled"
else
  fail "Session continuity not enabled"
fi

if grep -q "GUARDIAN_MODE" "$SENSOR_FILE"; then
  pass "Guardian mode environment variable configured"
else
  fail "Guardian mode environment variable missing"
fi

if grep -q "MERGE_STRATEGY" "$SENSOR_FILE"; then
  pass "Merge strategy environment variable configured"
else
  fail "Merge strategy environment variable missing"
fi

# Test 6: Check values.yaml configuration
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 6: Validate values.yaml Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

VALUES_FILE="$PROJECT_ROOT/infra/charts/controller/values.yaml"

if grep -q "guardianMode:" "$VALUES_FILE"; then
  pass "Guardian mode configuration present in values.yaml"
else
  fail "Guardian mode configuration missing from values.yaml"
fi

if grep -q "targetRepository: \"5dlabs/cto\"" "$VALUES_FILE"; then
  pass "Target repository set to 5dlabs/cto"
else
  fail "Target repository not set correctly"
fi

if grep -q "mergeStrategy: \"squash\"" "$VALUES_FILE"; then
  pass "Merge strategy set to squash"
else
  fail "Merge strategy not set to squash"
fi

if grep -q "autoMerge: true" "$VALUES_FILE"; then
  pass "Auto-merge enabled"
else
  fail "Auto-merge not enabled"
fi

# Test 7: Check cluster deployment (optional)
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 7: Check Cluster Deployment (Optional)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if kubectl get sensor atlas-pr-guardian -n argo &>/dev/null; then
  pass "Sensor deployed to cluster"
  
  # Check sensor status
  SENSOR_STATUS=$(kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.status.conditions[0].type}' 2>/dev/null || echo "Unknown")
  info "Sensor status: $SENSOR_STATUS"
  
  if [ "$SENSOR_STATUS" = "Active" ]; then
    pass "Sensor is active"
  else
    warn "Sensor is not active (status: $SENSOR_STATUS)"
  fi
else
  warn "Sensor not deployed to cluster (run after merging to main)"
fi

if kubectl get application atlas-pr-guardian-sensor -n argocd &>/dev/null; then
  pass "ArgoCD application exists"
  
  # Check sync status
  SYNC_STATUS=$(kubectl get application atlas-pr-guardian-sensor -n argocd -o jsonpath='{.status.sync.status}' 2>/dev/null || echo "Unknown")
  info "Sync status: $SYNC_STATUS"
  
  if [ "$SYNC_STATUS" = "Synced" ]; then
    pass "Application is synced"
  else
    warn "Application not synced (status: $SYNC_STATUS)"
  fi
else
  warn "ArgoCD application not deployed (run after merging to main)"
fi

# Test 8: Check GitHub webhook EventSource
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 8: Check GitHub Webhook EventSource (Optional)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if kubectl get eventsource github -n argo &>/dev/null; then
  pass "GitHub EventSource exists"
  
  # Check if it's running
  EVENTSOURCE_STATUS=$(kubectl get eventsource github -n argo -o jsonpath='{.status.conditions[0].type}' 2>/dev/null || echo "Unknown")
  info "EventSource status: $EVENTSOURCE_STATUS"
  
  if [ "$EVENTSOURCE_STATUS" = "Active" ]; then
    pass "EventSource is active"
  else
    warn "EventSource is not active (status: $EVENTSOURCE_STATUS)"
  fi
else
  warn "GitHub EventSource not found (required for sensor to work)"
fi

# Test 9: Validate documentation
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 9: Validate Documentation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

DOC_FILE="$PROJECT_ROOT/docs/engineering/atlas-pr-guardian.md"

if [ -f "$DOC_FILE" ]; then
  pass "Documentation file exists"
  
  # Check for key sections
  if grep -q "## Overview" "$DOC_FILE"; then
    pass "Documentation has Overview section"
  else
    warn "Documentation missing Overview section"
  fi
  
  if grep -q "## Architecture" "$DOC_FILE"; then
    pass "Documentation has Architecture section"
  else
    warn "Documentation missing Architecture section"
  fi
  
  if grep -q "## Deployment" "$DOC_FILE"; then
    pass "Documentation has Deployment section"
  else
    warn "Documentation missing Deployment section"
  fi
else
  fail "Documentation file not found: $DOC_FILE"
fi

# Summary
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    Test Summary                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
  echo -e "${GREEN}✅ All tests passed!${NC}"
  echo ""
  echo "Next steps:"
  echo "  1. Create feature branch: git checkout -b feature/atlas-pr-guardian"
  echo "  2. Commit changes: git add . && git commit -m 'feat(atlas): add PR guardian automation'"
  echo "  3. Push branch: git push -u origin feature/atlas-pr-guardian"
  echo "  4. Create PR and merge to main"
  echo "  5. ArgoCD will automatically deploy the sensor"
  echo "  6. Monitor deployment: kubectl get sensor atlas-pr-guardian -n argo"
  echo ""
  exit 0
else
  echo -e "${RED}❌ Some tests failed. Please fix the issues above.${NC}"
  echo ""
  exit 1
fi


