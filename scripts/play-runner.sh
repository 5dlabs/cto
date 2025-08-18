#!/bin/bash
# Play Runner - Sequential Project Execution
# Executes tasks in docs/.taskmaster/docs/ sequentially using play-workflow-template

set -e

# Configuration
NAMESPACE="${NAMESPACE:-agent-platform}"
DOCS_DIR="${DOCS_DIR:-docs/.taskmaster/docs}"
WORKFLOW_TEMPLATE="${WORKFLOW_TEMPLATE:-play-workflow-template}"
REPOSITORY="${REPOSITORY:-5dlabs/cto}"
DOCS_REPOSITORY="${DOCS_REPOSITORY:-https://github.com/5dlabs/cto}"
DOCS_PROJECT_DIR="${DOCS_PROJECT_DIR:-docs}"
DOCS_BRANCH="${DOCS_BRANCH:-main}"
GITHUB_APP_REX="${GITHUB_APP_REX:-5DLabs-Rex}"
GITHUB_APP_CLEO="${GITHUB_APP_CLEO:-5DLabs-Cleo}"
GITHUB_APP_TESS="${GITHUB_APP_TESS:-5DLabs-Tess}"

# Runtime options
START_FROM="${START_FROM:-1}"
LIMIT="${LIMIT:-}"
DRY_RUN="${DRY_RUN:-false}"
VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Help function
show_help() {
    cat << EOF
Play Runner - Sequential Project Execution

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --start-from <N>        Start from task N (default: 1)
    --limit <N>            Limit to first N tasks (for testing)
    --dry-run              Show what would be done without executing
    --verbose              Enable verbose logging
    --namespace <ns>       Kubernetes namespace (default: agent-platform)
    --help                 Show this help

ENVIRONMENT VARIABLES:
    NAMESPACE              Kubernetes namespace
    DOCS_DIR               TaskMaster docs directory
    WORKFLOW_TEMPLATE      Workflow template name
    REPOSITORY             Repository identifier
    DOCS_REPOSITORY        Docs repository URL
    DOCS_PROJECT_DIR       Docs project directory
    DOCS_BRANCH            Docs branch
    GITHUB_APP_REX         Rex GitHub App name
    GITHUB_APP_CLEO        Cleo GitHub App name  
    GITHUB_APP_TESS        Tess GitHub App name

EXAMPLES:
    $0                     # Run all tasks from task-1
    $0 --start-from 3      # Resume from task-3
    $0 --dry-run           # Show execution plan
    $0 --verbose           # Enable detailed logging

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --start-from)
            START_FROM="$2"
            shift 2
            ;;
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is required but not installed"
        exit 1
    fi
    
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_error "Namespace '$NAMESPACE' does not exist"
        exit 1
    fi
    
    if ! kubectl get workflowtemplate "$WORKFLOW_TEMPLATE" -n "$NAMESPACE" &> /dev/null; then
        log_error "WorkflowTemplate '$WORKFLOW_TEMPLATE' not found in namespace '$NAMESPACE'"
        exit 1
    fi
    
    if [[ ! -d "$DOCS_DIR" ]]; then
        log_error "TaskMaster docs directory '$DOCS_DIR' does not exist"
        exit 1
    fi
    
    log_success "All dependencies validated"
}

# Discover available tasks in ascending numeric order
discover_tasks() {
    log_verbose "Discovering tasks in $DOCS_DIR..."
    
    local tasks=()
    while IFS= read -r -d '' task_dir; do
        local task_name=$(basename "$task_dir")
        if [[ "$task_name" =~ ^task-([0-9]+)$ ]]; then
            local task_id="${BASH_REMATCH[1]}"
            # Only include tasks >= START_FROM
            if (( task_id >= START_FROM )); then
                tasks+=("$task_id")
            fi
        fi
    done < <(find "$DOCS_DIR" -maxdepth 1 -type d -name "task-*" -print0)
    
    # Sort numerically
    IFS=$'\n' tasks=($(sort -n <<<"${tasks[*]}"))
    unset IFS
    
    # Apply limit if specified
    if [[ -n "$LIMIT" ]] && (( LIMIT > 0 )) && (( ${#tasks[@]} > LIMIT )); then
        tasks=("${tasks[@]:0:$LIMIT}")
    fi
    
    if [[ ${#tasks[@]} -eq 0 ]]; then
        log_error "No tasks found starting from task-$START_FROM"
        exit 1
    fi
    
    log_verbose "Found ${#tasks[@]} tasks: ${tasks[*]}"
    echo "${tasks[@]}"
}

# Generate workflow manifest for a task
generate_workflow_manifest() {
    local task_id="$1"
    local workflow_name="play-task-${task_id}-workflow"
    
    cat << EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: ${workflow_name}-
  namespace: ${NAMESPACE}
  labels:
    task-id: "${task_id}"
    repository: "$(echo "$REPOSITORY" | tr '/' '-')"
    current-stage: "pending"
    play-runner: "true"
spec:
  workflowTemplateRef:
    name: ${WORKFLOW_TEMPLATE}
  arguments:
    parameters:
      - name: task-id
        value: "${task_id}"
      - name: repository
        value: "${REPOSITORY}"
      - name: docs-repository
        value: "${DOCS_REPOSITORY}"
      - name: docs-project-directory
        value: "${DOCS_PROJECT_DIR}"
      - name: docs-branch
        value: "${DOCS_BRANCH}"
      - name: github-app-rex
        value: "${GITHUB_APP_REX}"
      - name: github-app-cleo
        value: "${GITHUB_APP_CLEO}"
      - name: github-app-tess
        value: "${GITHUB_APP_TESS}"
EOF
}

# Submit workflow for a task
submit_workflow() {
    local task_id="$1"
    
    log_info "Submitting workflow for task-$task_id..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would submit workflow:"
        generate_workflow_manifest "$task_id" | sed 's/^/  /'
        echo "play-task-${task_id}-workflow-dryrun"
        return 0
    fi
    
    local manifest_file="/tmp/play-task-${task_id}-workflow.yaml"
    generate_workflow_manifest "$task_id" > "$manifest_file"
    
    local workflow_name
    workflow_name=$(kubectl create -f "$manifest_file" -o jsonpath='{.metadata.name}')
    
    if [[ -z "$workflow_name" ]]; then
        log_error "Failed to create workflow for task-$task_id"
        return 1
    fi
    
    log_success "Created workflow: $workflow_name"
    echo "$workflow_name"
}

# Wait for workflow completion
wait_for_completion() {
    local workflow_name="$1"
    local task_id="$2"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would wait for workflow completion: $workflow_name"
        return 0
    fi
    
    log_info "Waiting for workflow completion: $workflow_name"
    
    local timeout=3600  # 1 hour timeout
    local elapsed=0
    local check_interval=30
    
    while (( elapsed < timeout )); do
        local status phase
        status=$(kubectl get workflow "$workflow_name" -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
        
        case "$status" in
            "Succeeded")
                log_success "Task-$task_id completed successfully"
                return 0
                ;;
            "Failed"|"Error")
                log_error "Task-$task_id failed"
                kubectl get workflow "$workflow_name" -n "$NAMESPACE" -o yaml | tail -20
                return 1
                ;;
            "Running")
                log_verbose "Task-$task_id still running... (${elapsed}s elapsed)"
                ;;
            "NotFound")
                log_error "Workflow $workflow_name not found"
                return 1
                ;;
            *)
                log_verbose "Task-$task_id status: $status (${elapsed}s elapsed)"
                ;;
        esac
        
        sleep $check_interval
        elapsed=$((elapsed + check_interval))
    done
    
    log_error "Timeout waiting for task-$task_id completion (${timeout}s)"
    return 1
}

# Check if any workflows are currently running
check_concurrent_workflows() {
    if [[ "$DRY_RUN" == "true" ]]; then
        return 0
    fi
    
    local running_workflows
    running_workflows=$(kubectl get workflows -n "$NAMESPACE" -l play-runner=true --field-selector status.phase=Running -o name 2>/dev/null | wc -l)
    
    if (( running_workflows > 0 )); then
        log_warning "Found $running_workflows running Play workflows. Waiting for completion..."
        
        # Wait for existing workflows to complete
        while (( running_workflows > 0 )); do
            sleep 30
            running_workflows=$(kubectl get workflows -n "$NAMESPACE" -l play-runner=true --field-selector status.phase=Running -o name 2>/dev/null | wc -l)
            log_verbose "Still waiting for $running_workflows workflows to complete..."
        done
        
        log_success "All previous workflows completed"
    fi
}

# Main execution function
main() {
    echo "=================================="
    echo "🎯 Play Runner - Sequential Execution"
    echo "=================================="
    echo ""
    
    log_info "Configuration:"
    log_info "  Namespace: $NAMESPACE"
    log_info "  Docs Directory: $DOCS_DIR"
    log_info "  Workflow Template: $WORKFLOW_TEMPLATE"
    log_info "  Repository: $REPOSITORY"
    log_info "  Start From: task-$START_FROM"
    log_info "  Dry Run: $DRY_RUN"
    echo ""
    
    # Validate environment
    check_dependencies
    
    # Discover tasks
    local tasks_output
    tasks_output=$(discover_tasks)
    local tasks=($tasks_output)
    
    log_info "Discovered ${#tasks[@]} tasks: ${tasks[*]}"
    echo ""
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN - Execution Plan:"
        for task_id in "${tasks[@]}"; do
            log_info "  → Would execute task-$task_id"
        done
        echo ""
        log_info "Use --verbose to see workflow manifests"
        exit 0
    fi
    
    # Check for concurrent workflows
    check_concurrent_workflows
    
    # Execute tasks sequentially
    local total_tasks=${#tasks[@]}
    local completed_tasks=0
    local failed_tasks=0
    local start_time=$(date +%s)
    
    log_info "Starting sequential execution of $total_tasks tasks..."
    echo ""
    
    for task_id in "${tasks[@]}"; do
        echo "=================================="
        log_info "📋 Executing Task $task_id ($(( completed_tasks + 1 ))/$total_tasks)"
        echo "=================================="
        
        # Verify task directory exists
        if [[ ! -d "$DOCS_DIR/task-$task_id" ]]; then
            log_error "Task directory not found: $DOCS_DIR/task-$task_id"
            ((failed_tasks++))
            continue
        fi
        
        # Submit workflow
        local workflow_name
        if ! workflow_name=$(submit_workflow "$task_id"); then
            log_error "Failed to submit workflow for task-$task_id"
            ((failed_tasks++))
            continue
        fi
        
        # Wait for completion
        if wait_for_completion "$workflow_name" "$task_id"; then
            ((completed_tasks++))
            log_success "✅ Task-$task_id completed successfully"
        else
            ((failed_tasks++))
            log_error "❌ Task-$task_id failed"
            
            # Ask user if they want to continue
            echo ""
            read -p "Continue with remaining tasks? (y/N): " -r
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Execution stopped by user"
                break
            fi
        fi
        
        echo ""
    done
    
    # Summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo "=================================="
    echo "📊 Execution Summary"
    echo "=================================="
    log_info "Total Tasks: $total_tasks"
    log_success "Completed: $completed_tasks"
    if (( failed_tasks > 0 )); then
        log_error "Failed: $failed_tasks"
    fi
    log_info "Duration: ${duration}s"
    echo ""
    
    if (( failed_tasks > 0 )); then
        log_error "Play Runner completed with failures"
        exit 1
    else
        log_success "🎉 All tasks completed successfully!"
        exit 0
    fi
}

# Run main function
main "$@"
