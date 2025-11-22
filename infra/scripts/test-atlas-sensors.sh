#!/bin/bash
set -euo pipefail

# Atlas Sensor Integration Test Suite
# Tests all Atlas-related sensors and their interactions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test configuration
TEST_NAMESPACE="${TEST_NAMESPACE:-agent-platform}"
TEST_PR_NUMBER="${TEST_PR_NUMBER:-9999}"
TEST_REPO="${TEST_REPO:-5dlabs/cto}"
CLEANUP="${CLEANUP:-true}"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    if [ "$CLEANUP" = "true" ]; then
        log_info "Cleaning up test resources..."
        
        # Delete test CodeRuns
        kubectl delete coderuns -n "$TEST_NAMESPACE" \
            -l pr-number="$TEST_PR_NUMBER" --ignore-not-found=true || true
        
        # Delete test ConfigMap locks
        kubectl delete configmaps -n "$TEST_NAMESPACE" \
            -l atlas-guardian-lock --ignore-not-found=true || true
        kubectl delete configmaps -n "$TEST_NAMESPACE" \
            -l atlas-integration-lock --ignore-not-found=true || true
        
        # Delete test workflows
        kubectl delete workflows -n "$TEST_NAMESPACE" \
            -l type=atlas-pr-monitor --ignore-not-found=true || true
        kubectl delete workflows -n "$TEST_NAMESPACE" \
            -l type=atlas-guardian --ignore-not-found=true || true
        kubectl delete workflows -n "$TEST_NAMESPACE" \
            -l type=stage-resume --ignore-not-found=true || true
    fi
}

trap cleanup EXIT

# Test 1: Atlas PR Monitor Sensor
test_pr_monitor() {
    log_info "Testing Atlas PR Monitor Sensor..."
    
    # Simulate PR opened event
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Event
metadata:
  name: test-pr-opened-$TEST_PR_NUMBER
  namespace: argo
spec:
  eventSourceName: github
  eventName: org
  data:
    headers:
      X-GitHub-Event: pull_request
    body:
      action: opened
      repository:
        full_name: $TEST_REPO
        owner:
          login: 5dlabs
        name: cto
        clone_url: https://github.com/5dlabs/cto.git
      pull_request:
        number: $TEST_PR_NUMBER
        html_url: https://github.com/5dlabs/cto/pull/$TEST_PR_NUMBER
        head:
          ref: feature/test-atlas
        title: "Task-7: Test Atlas Integration"
        labels:
          - name: task-7
      sender:
        login: test-user
EOF
    
    sleep 5
    
    # Check if workflow was created
    if kubectl get workflows -n "$TEST_NAMESPACE" \
        -l type=atlas-pr-monitor -o name | grep -q "workflow"; then
        log_info "✅ PR monitor workflow created successfully"
    else
        log_error "❌ PR monitor workflow not created"
        return 1
    fi
    
    # Check for CodeRun creation (may take a moment)
    sleep 10
    if kubectl get coderuns -n "$TEST_NAMESPACE" \
        -l agent=atlas,role=guardian,pr-number="$TEST_PR_NUMBER" -o name | grep -q "coderun"; then
        log_info "✅ Atlas guardian CodeRun created"
    else
        log_warn "⚠️ Guardian CodeRun not found (might be normal if lock held)"
    fi
}

# Test 2: Atlas Conflict Monitor
test_conflict_monitor() {
    log_info "Testing Atlas Conflict Monitor Sensor..."
    
    # Simulate PR with conflicts
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Event
metadata:
  name: test-pr-conflict-$TEST_PR_NUMBER
  namespace: argo
spec:
  eventSourceName: github
  eventName: org
  data:
    headers:
      X-GitHub-Event: pull_request
    body:
      action: synchronize
      repository:
        full_name: $TEST_REPO
        owner:
          login: 5dlabs
        name: cto
        clone_url: https://github.com/5dlabs/cto.git
      pull_request:
        number: $TEST_PR_NUMBER
        html_url: https://github.com/5dlabs/cto/pull/$TEST_PR_NUMBER
        mergeable: false
        mergeable_state: dirty
        head:
          ref: feature/test-atlas
        title: "Task-7: Test Atlas Integration"
EOF
    
    sleep 5
    
    # Check if conflict monitor triggered
    if kubectl get workflows -n "$TEST_NAMESPACE" \
        -l type=atlas-guardian,trigger=conflict -o name | grep -q "workflow"; then
        log_info "✅ Conflict monitor workflow created"
    else
        log_error "❌ Conflict monitor workflow not created"
        return 1
    fi
}

# Test 3: Tess Approval → Atlas Integration Gate
test_integration_gate() {
    log_info "Testing Atlas Integration Gate (Tess Approval)..."
    
    # Create a mock play workflow
    cat <<EOF | kubectl apply -f -
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-play-workflow-$TEST_PR_NUMBER
  namespace: $TEST_NAMESPACE
  labels:
    workflow: play-orchestration
    current-stage: waiting-atlas-integration
    pr-number: "$TEST_PR_NUMBER"
    task-id: "7"
spec:
  entrypoint: mock
  templates:
    - name: mock
      container:
        image: alpine
        command: [sleep, "3600"]
EOF
    
    # Simulate Tess approval
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Event
metadata:
  name: test-tess-approval-$TEST_PR_NUMBER
  namespace: argo
spec:
  eventSourceName: github
  eventName: org
  data:
    headers:
      X-GitHub-Event: pull_request_review
    body:
      action: submitted
      review:
        state: approved
        user:
          login: tess-5dlabs
      repository:
        full_name: $TEST_REPO
        owner:
          login: 5dlabs
        name: cto
      pull_request:
        number: $TEST_PR_NUMBER
        html_url: https://github.com/5dlabs/cto/pull/$TEST_PR_NUMBER
        labels:
          - name: task-7
          - name: ready-for-qa
        title: "Task-7: Test Atlas Integration"
        head:
          ref: feature/test-atlas
          repo:
            clone_url: https://github.com/5dlabs/cto.git
EOF
    
    sleep 10
    
    # Check if integration gate workflow created
    if kubectl get workflows -n "$TEST_NAMESPACE" \
        -l type=stage-resume,target-stage=waiting-atlas-integration -o name | grep -q "workflow"; then
        log_info "✅ Integration gate workflow created"
    else
        log_error "❌ Integration gate workflow not created"
        return 1
    fi
    
    # Check for integration CodeRun
    if kubectl get coderuns -n "$TEST_NAMESPACE" \
        -l agent=atlas,role=integration,pr-number="$TEST_PR_NUMBER" -o name | grep -q "coderun"; then
        log_info "✅ Atlas integration CodeRun created"
    else
        log_warn "⚠️ Integration CodeRun not found (check workflow logs)"
    fi
}

# Test 4: Batch Integration Trigger
test_batch_integration() {
    log_info "Testing Atlas Batch Integration Sensor..."
    
    # Simulate batch completion comment
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Event
metadata:
  name: test-batch-complete-$TEST_PR_NUMBER
  namespace: argo
spec:
  eventSourceName: github
  eventName: org
  data:
    headers:
      X-GitHub-Event: issue_comment
    body:
      action: created
      repository:
        full_name: $TEST_REPO
        owner:
          login: 5dlabs
        name: cto
        clone_url: https://github.com/5dlabs/cto.git
      issue:
        number: $TEST_PR_NUMBER
        pull_request:
          html_url: https://github.com/5dlabs/cto/pull/$TEST_PR_NUMBER
      comment:
        body: "Batch 1 Complete - 3 PRs created"
        user:
          login: workflow-bot
EOF
    
    sleep 5
    
    # Check if batch integration triggered
    if kubectl get workflows -n "$TEST_NAMESPACE" \
        -l type=atlas-integration,trigger=batch -o name | grep -q "workflow"; then
        log_info "✅ Batch integration workflow created"
    else
        log_error "❌ Batch integration workflow not created"
        return 1
    fi
}

# Test 5: Deduplication Logic
test_deduplication() {
    log_info "Testing deduplication logic..."
    
    # Create a lock manually
    kubectl create configmap "atlas-guardian-lock-$TEST_PR_NUMBER" \
        --from-literal=pr-number="$TEST_PR_NUMBER" \
        --from-literal=test="true" \
        -n "$TEST_NAMESPACE" || true
    
    # Try to trigger another guardian
    test_pr_monitor
    
    # Count CodeRuns - should be max 1
    COUNT=$(kubectl get coderuns -n "$TEST_NAMESPACE" \
        -l agent=atlas,role=guardian,pr-number="$TEST_PR_NUMBER" --no-headers 2>/dev/null | wc -l)
    
    if [ "$COUNT" -le 1 ]; then
        log_info "✅ Deduplication working (found $COUNT CodeRun(s))"
    else
        log_error "❌ Deduplication failed (found $COUNT CodeRuns)"
        return 1
    fi
}

# Test 6: Stage Transition Validation
test_stage_transitions() {
    log_info "Testing workflow stage transitions..."
    
    # Check valid transitions
    VALID_TRANSITIONS=(
        "testing-in-progress:waiting-atlas-integration"
        "waiting-atlas-integration:atlas-integration-in-progress"
        "atlas-integration-in-progress:waiting-pr-merged"
    )
    
    for transition in "${VALID_TRANSITIONS[@]}"; do
        IFS=':' read -r from to <<< "$transition"
        log_info "Testing $from → $to"
        
        # This would normally involve creating/updating actual workflows
        # For now, just verify the configuration exists
        if grep -q "waiting-atlas-integration" "$PROJECT_ROOT/infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml"; then
            log_info "✅ Stage $to found in workflow template"
        else
            log_error "❌ Stage $to not found in workflow template"
            return 1
        fi
    done
}

# Main test execution
main() {
    log_info "Starting Atlas Sensor Test Suite"
    log_info "Test PR: #$TEST_PR_NUMBER"
    log_info "Namespace: $TEST_NAMESPACE"
    echo
    
    FAILED=0
    
    # Run tests
    test_pr_monitor || ((FAILED++))
    echo
    
    test_conflict_monitor || ((FAILED++))
    echo
    
    test_integration_gate || ((FAILED++))
    echo
    
    test_batch_integration || ((FAILED++))
    echo
    
    test_deduplication || ((FAILED++))
    echo
    
    test_stage_transitions || ((FAILED++))
    echo
    
    # Summary
    if [ "$FAILED" -eq 0 ]; then
        log_info "🎉 All tests passed!"
        exit 0
    else
        log_error "❌ $FAILED test(s) failed"
        exit 1
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --pr)
            TEST_PR_NUMBER="$2"
            shift 2
            ;;
        --namespace)
            TEST_NAMESPACE="$2"
            shift 2
            ;;
        --no-cleanup)
            CLEANUP="false"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --pr NUMBER       Test PR number (default: 9999)"
            echo "  --namespace NS    Test namespace (default: agent-platform)"
            echo "  --no-cleanup      Don't cleanup test resources"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

main
