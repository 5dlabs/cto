#!/bin/bash
set -euo pipefail

# CI Failure Remediation System - Comprehensive Test Script
# Tests the end-to-end CI failure detection and remediation workflow

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_BRANCH="test/ci-remediation-$(date +%s)"
NAMESPACE="cto"
SENSOR_NAME="ci-failure-remediation"

echo "════════════════════════════════════════════════════════════════"
echo "  CI Failure Remediation System - Test Suite"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "info")
            echo -e "${BLUE}ℹ${NC} $message"
            ;;
        "success")
            echo -e "${GREEN}✓${NC} $message"
            ;;
        "warning")
            echo -e "${YELLOW}⚠${NC} $message"
            ;;
        "error")
            echo -e "${RED}✗${NC} $message"
            ;;
    esac
}

# Function to check prerequisites
check_prerequisites() {
    print_status "info" "Checking prerequisites..."
    
    local missing_tools=()
    
    for tool in kubectl gh jq; do
        if ! command -v $tool &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        print_status "error" "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    # Check kubectl access
    if ! kubectl get ns $NAMESPACE &> /dev/null; then
        print_status "error" "Cannot access namespace: $NAMESPACE"
        exit 1
    fi
    
    # Check GitHub CLI authentication
    if ! gh auth status &> /dev/null; then
        print_status "error" "GitHub CLI not authenticated. Run: gh auth login"
        exit 1
    fi
    
    print_status "success" "All prerequisites met"
}

# Function to check sensor deployment
check_sensor_deployment() {
    print_status "info" "Checking sensor deployment..."
    
    if ! kubectl get sensor $SENSOR_NAME -n argo &> /dev/null; then
        print_status "error" "Sensor '$SENSOR_NAME' not found in namespace 'argo'"
        print_status "info" "Deploy with: kubectl apply -f infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml"
        exit 1
    fi
    
    # Check sensor status
    local sensor_status=$(kubectl get sensor $SENSOR_NAME -n argo -o jsonpath='{.status.phase}')
    if [ "$sensor_status" != "Running" ]; then
        print_status "warning" "Sensor status: $sensor_status (expected: Running)"
    else
        print_status "success" "Sensor is running"
    fi
    
    # Check sensor pod
    local sensor_pod=$(kubectl get pods -n argo -l sensor-name=$SENSOR_NAME -o jsonpath='{.items[0].metadata.name}')
    if [ -n "$sensor_pod" ]; then
        local pod_status=$(kubectl get pod $sensor_pod -n argo -o jsonpath='{.status.phase}')
        print_status "success" "Sensor pod: $sensor_pod (status: $pod_status)"
    else
        print_status "warning" "Sensor pod not found"
    fi
}

# Function to check EventSource
check_eventsource() {
    print_status "info" "Checking GitHub EventSource..."
    
    if ! kubectl get eventsource github -n argo &> /dev/null; then
        print_status "error" "EventSource 'github' not found in namespace 'argo'"
        exit 1
    fi
    
    local es_status=$(kubectl get eventsource github -n argo -o jsonpath='{.status.phase}')
    if [ "$es_status" != "Running" ]; then
        print_status "warning" "EventSource status: $es_status (expected: Running)"
    else
        print_status "success" "EventSource is running"
    fi
}

# Function to simulate CI failure
simulate_ci_failure() {
    print_status "info" "Simulating CI failure..."
    
    # Create test branch
    print_status "info" "Creating test branch: $TEST_BRANCH"
    git checkout -b $TEST_BRANCH
    
    # Introduce a Clippy error in a Rust file
    local test_file="crates/controller/src/lib.rs"
    print_status "info" "Introducing Clippy error in $test_file"
    
    # Backup original file
    cp $test_file ${test_file}.backup
    
    # Add code that will fail Clippy pedantic
    cat >> $test_file << 'EOF'

// This will fail Clippy pedantic checks
#[allow(dead_code)]
fn test_ci_remediation_trigger() {
    let unused_var = 42;
    println!("Testing CI remediation");
}
EOF
    
    # Commit and push
    git add $test_file
    git commit -m "test: trigger CI failure for remediation testing

This commit intentionally introduces a Clippy error to test
the CI failure remediation system.

[test-ci-remediation]"
    
    print_status "info" "Pushing test branch..."
    git push origin $TEST_BRANCH
    
    print_status "success" "Test branch pushed: $TEST_BRANCH"
    
    # Get the workflow run ID
    print_status "info" "Waiting for workflow to start..."
    sleep 10
    
    local run_id=$(gh run list --branch $TEST_BRANCH --limit 1 --json databaseId --jq '.[0].databaseId')
    if [ -n "$run_id" ]; then
        print_status "success" "Workflow run started: $run_id"
        echo "  View at: https://github.com/5dlabs/cto/actions/runs/$run_id"
    else
        print_status "warning" "Could not find workflow run ID"
    fi
    
    # Restore original file
    mv ${test_file}.backup $test_file
    git checkout main
}

# Function to monitor remediation
monitor_remediation() {
    local timeout=600  # 10 minutes
    local elapsed=0
    local check_interval=15
    
    print_status "info" "Monitoring for remediation CodeRun..."
    print_status "info" "Timeout: ${timeout}s, checking every ${check_interval}s"
    
    while [ $elapsed -lt $timeout ]; do
        # Check for CodeRun with ci-remediation role
        local coderuns=$(kubectl get coderun -n $NAMESPACE \
            -l role=ci-remediation \
            --sort-by=.metadata.creationTimestamp \
            -o json)
        
        local count=$(echo "$coderuns" | jq '.items | length')
        
        if [ "$count" -gt 0 ]; then
            print_status "success" "Found $count remediation CodeRun(s)"
            
            # Get the latest one
            local latest=$(echo "$coderuns" | jq -r '.items[-1]')
            local name=$(echo "$latest" | jq -r '.metadata.name')
            local workflow_name=$(echo "$latest" | jq -r '.metadata.labels["workflow-name"] // "unknown"')
            local status=$(echo "$latest" | jq -r '.status.phase // "Pending"')
            
            print_status "info" "CodeRun: $name"
            print_status "info" "Workflow: $workflow_name"
            print_status "info" "Status: $status"
            
            # Get pod logs if available
            local pod=$(kubectl get pods -n $NAMESPACE -l coderun=$name -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
            if [ -n "$pod" ]; then
                print_status "info" "Pod: $pod"
                print_status "info" "Tailing logs (last 20 lines)..."
                kubectl logs -n $NAMESPACE $pod --tail=20 || true
            fi
            
            return 0
        fi
        
        echo -n "."
        sleep $check_interval
        elapsed=$((elapsed + check_interval))
    done
    
    print_status "warning" "Timeout waiting for remediation CodeRun"
    return 1
}

# Function to check sensor logs
check_sensor_logs() {
    print_status "info" "Checking sensor logs for recent activity..."
    
    local sensor_pod=$(kubectl get pods -n argo -l sensor-name=$SENSOR_NAME -o jsonpath='{.items[0].metadata.name}')
    if [ -n "$sensor_pod" ]; then
        print_status "info" "Recent sensor logs:"
        kubectl logs -n argo $sensor_pod --tail=50 | grep -i "workflow\|trigger\|failure" || true
    else
        print_status "warning" "Sensor pod not found"
    fi
}

# Function to cleanup test resources
cleanup_test() {
    print_status "info" "Cleaning up test resources..."
    
    # Delete test branch
    if git show-ref --verify --quiet refs/heads/$TEST_BRANCH; then
        print_status "info" "Deleting local test branch: $TEST_BRANCH"
        git branch -D $TEST_BRANCH 2>/dev/null || true
    fi
    
    # Delete remote test branch
    if gh api repos/5dlabs/cto/git/refs/heads/$TEST_BRANCH &> /dev/null; then
        print_status "info" "Deleting remote test branch: $TEST_BRANCH"
        gh api -X DELETE repos/5dlabs/cto/git/refs/heads/$TEST_BRANCH || true
    fi
    
    # Optionally delete test CodeRuns (commented out by default)
    # kubectl delete coderun -n $NAMESPACE -l role=ci-remediation --field-selector metadata.name~=test
    
    print_status "success" "Cleanup complete"
}

# Function to run dry-run test (no actual CI failure)
dry_run_test() {
    print_status "info" "Running dry-run test (checking configuration only)..."
    
    # Validate sensor YAML
    print_status "info" "Validating sensor YAML..."
    kubectl apply --dry-run=client -f $REPO_ROOT/infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml
    print_status "success" "Sensor YAML is valid"
    
    # Check template rendering
    print_status "info" "Checking agent template..."
    if grep -q "REMEDIATION_MODE.*ci-failure" $REPO_ROOT/infra/charts/controller/templates/code/claude/container-rex.sh.hbs; then
        print_status "success" "CI remediation mode found in Rex template"
    else
        print_status "error" "CI remediation mode NOT found in Rex template"
        exit 1
    fi
    
    print_status "success" "Dry-run test passed"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Test the CI Failure Remediation System

OPTIONS:
    --dry-run           Run configuration validation only (no actual CI failure)
    --check-only        Check deployment status without running tests
    --simulate          Simulate a CI failure and monitor remediation
    --cleanup           Clean up test resources
    --monitor           Monitor for existing remediation activity
    --help              Show this help message

EXAMPLES:
    # Check deployment status
    $0 --check-only

    # Run dry-run validation
    $0 --dry-run

    # Simulate CI failure and monitor remediation
    $0 --simulate

    # Monitor for existing remediation activity
    $0 --monitor

    # Clean up test resources
    $0 --cleanup

EOF
}

# Main execution
main() {
    local mode="full"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                mode="dry-run"
                shift
                ;;
            --check-only)
                mode="check-only"
                shift
                ;;
            --simulate)
                mode="simulate"
                shift
                ;;
            --cleanup)
                mode="cleanup"
                shift
                ;;
            --monitor)
                mode="monitor"
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_status "error" "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Execute based on mode
    case $mode in
        "dry-run")
            check_prerequisites
            dry_run_test
            ;;
        "check-only")
            check_prerequisites
            check_eventsource
            check_sensor_deployment
            check_sensor_logs
            ;;
        "simulate")
            check_prerequisites
            check_eventsource
            check_sensor_deployment
            simulate_ci_failure
            monitor_remediation
            ;;
        "monitor")
            check_prerequisites
            monitor_remediation
            check_sensor_logs
            ;;
        "cleanup")
            cleanup_test
            ;;
        "full")
            check_prerequisites
            check_eventsource
            check_sensor_deployment
            dry_run_test
            print_status "info" "Full test requires manual CI failure simulation"
            print_status "info" "Run with --simulate to trigger actual CI failure test"
            ;;
    esac
    
    echo ""
    print_status "success" "Test complete!"
}

# Run main function
main "$@"

