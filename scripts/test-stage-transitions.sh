#!/bin/bash
# Test Script for Workflow Stage Transitions (Task 7)
# This script validates atomic label updates and stage progression

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
NAMESPACE="argo"
TEST_TIMEOUT=300
WORKFLOW_NAME=""

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to create test workflow
create_test_workflow() {
    log_info "Creating test workflow for stage transitions..."
    
    echo "📋 Checking for stage-transitions-template..."
    if kubectl get workflowtemplate stage-transitions-template -n "$NAMESPACE" &>/dev/null; then
        echo "✅ Template exists in cluster"
    else
        echo "❌ Template not found. Applying from local file..."
        kubectl apply -f infra/charts/controller/templates/stage-transitions-template.yaml -n "$NAMESPACE"
    fi
    
    # Submit the test workflow
    WORKFLOW_NAME=$(argo submit infra/examples/test-stage-transitions.yaml \
        -n $NAMESPACE \
        --wait=false \
        -o name | cut -d'/' -f2)
    
    if [ -z "$WORKFLOW_NAME" ]; then
        log_error "Failed to create test workflow"
        exit 1
    fi
    
    log_success "Created workflow: $WORKFLOW_NAME"
}

# Function to test atomic label update
test_atomic_update() {
    local stage="$1"
    log_info "Testing atomic update to stage: $stage"
    
    # Apply the patch
    if kubectl patch workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"'$stage'","updated-at":"'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"}}}'
    then
        log_success "Patch applied successfully"
    else
        log_error "Failed to apply patch"
        return 1
    fi
    
    # Verify the update
    CURRENT_STAGE=$(kubectl get workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        -o jsonpath='{.metadata.labels.current-stage}')
    
    if [ "$CURRENT_STAGE" = "$stage" ]; then
        log_success "Stage verified: $CURRENT_STAGE"
        return 0
    else
        log_error "Stage mismatch. Expected: $stage, Got: $CURRENT_STAGE"
        return 1
    fi
}

# Function to test concurrent updates
test_concurrent_updates() {
    log_info "Testing concurrent stage updates (race condition simulation)..."
    
    # Start multiple concurrent updates
    (kubectl patch workflow "$WORKFLOW_NAME" -n $NAMESPACE --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"concurrent-1"}}}' &)
    
    (kubectl patch workflow "$WORKFLOW_NAME" -n $NAMESPACE --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"concurrent-2"}}}' &)
    
    (kubectl patch workflow "$WORKFLOW_NAME" -n $NAMESPACE --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"concurrent-3"}}}' &)
    
    # Wait for all background jobs
    wait
    
    # Check final state
    FINAL_STAGE=$(kubectl get workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        -o jsonpath='{.metadata.labels.current-stage}')
    
    log_info "Final stage after concurrent updates: $FINAL_STAGE"
    
    # Verify one of the updates won
    if [[ "$FINAL_STAGE" =~ concurrent-[123] ]]; then
        log_success "Concurrent updates handled correctly. Winner: $FINAL_STAGE"
        return 0
    else
        log_error "Unexpected stage after concurrent updates: $FINAL_STAGE"
        return 1
    fi
}

# Function to test stage progression
test_stage_progression() {
    log_info "Testing complete stage progression flow..."
    
    local stages=("waiting-pr-created" "waiting-ready-for-qa" "waiting-pr-approved" "completed")
    
    for stage in "${stages[@]}"; do
        if test_atomic_update "$stage"; then
            log_success "Stage progression to '$stage' successful"
            sleep 1  # Brief pause between transitions
        else
            log_error "Stage progression to '$stage' failed"
            return 1
        fi
    done
    
    log_success "Complete stage progression tested successfully"
}

# Function to test error recovery
test_error_recovery() {
    log_info "Testing error recovery mechanisms..."
    
    # Try to set an invalid stage (should still work with atomic updates)
    kubectl patch workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"invalid-stage-test"}}}'
    
    # Recover to a valid stage
    if test_atomic_update "waiting-pr-created"; then
        log_success "Error recovery successful"
    else
        log_error "Error recovery failed"
        return 1
    fi
}

# Function to test label persistence
test_label_persistence() {
    log_info "Testing label persistence across updates..."
    
    # Set initial labels
    kubectl patch workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        --type='merge' \
        --patch='{
            "metadata": {
                "labels": {
                    "current-stage": "test-stage",
                    "task-id": "7",
                    "repository": "5dlabs/cto",
                    "custom-label": "should-persist"
                }
            }
        }'
    
    # Update only the stage
    kubectl patch workflow "$WORKFLOW_NAME" \
        -n $NAMESPACE \
        --type='merge' \
        --patch='{"metadata":{"labels":{"current-stage":"new-test-stage"}}}'
    
    # Verify other labels persisted
    TASK_ID=$(kubectl get workflow "$WORKFLOW_NAME" -n $NAMESPACE -o jsonpath='{.metadata.labels.task-id}')
    CUSTOM_LABEL=$(kubectl get workflow "$WORKFLOW_NAME" -n $NAMESPACE -o jsonpath='{.metadata.labels.custom-label}')
    
    if [ "$TASK_ID" = "7" ] && [ "$CUSTOM_LABEL" = "should-persist" ]; then
        log_success "Label persistence verified"
    else
        log_error "Labels did not persist correctly"
        return 1
    fi
}

# Function to cleanup test resources
cleanup() {
    log_info "Cleaning up test resources..."
    
    if [ -n "$WORKFLOW_NAME" ]; then
        kubectl delete workflow "$WORKFLOW_NAME" -n $NAMESPACE --ignore-not-found=true
        log_success "Cleaned up workflow: $WORKFLOW_NAME"
    fi
}

# Main test execution
main() {
    echo "========================================="
    echo "Workflow Stage Transition Test Suite"
    echo "========================================="
    echo ""
    
    # Trap cleanup on exit
    trap cleanup EXIT
    
    # Run test suite
    log_info "Starting stage transition tests..."
    
    # Test 1: Create test workflow
    create_test_workflow
    
    # Test 2: Basic atomic updates
    log_info "Test 2: Basic atomic label updates"
    test_atomic_update "initial-test"
    
    # Test 3: Stage progression
    log_info "Test 3: Complete stage progression"
    test_stage_progression
    
    # Test 4: Concurrent updates
    log_info "Test 4: Concurrent update handling"
    test_concurrent_updates
    
    # Test 5: Error recovery
    log_info "Test 5: Error recovery"
    test_error_recovery
    
    # Test 6: Label persistence
    log_info "Test 6: Label persistence"
    test_label_persistence
    
    # Final summary
    echo ""
    echo "========================================="
    log_success "All stage transition tests PASSED!"
    echo "========================================="
    echo ""
    
    # Show final workflow state
    log_info "Final workflow labels:"
    kubectl get workflow "$WORKFLOW_NAME" -n $NAMESPACE -o jsonpath='{.metadata.labels}'
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Please install kubectl."
        exit 1
    fi
    
    # Check argo CLI
    if ! command -v argo &> /dev/null; then
        log_warning "argo CLI not found. Some tests may be limited."
    fi
    
    # Check cluster connection
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    # Check namespace exists
    if ! kubectl get namespace $NAMESPACE &> /dev/null; then
        log_error "Namespace $NAMESPACE does not exist"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Script entry point
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [--cleanup-only]"
    echo ""
    echo "Options:"
    echo "  --cleanup-only    Only cleanup existing test resources"
    echo "  --help, -h        Show this help message"
    exit 0
fi

if [ "${1:-}" = "--cleanup-only" ]; then
    log_info "Running cleanup only..."
    kubectl delete workflow -l test-type=stage-transitions -n $NAMESPACE --ignore-not-found=true
    log_success "Cleanup completed"
    exit 0
fi

# Run the tests
check_prerequisites
main