#!/bin/bash
# Workflow Monitor - Comprehensive Workflow Execution Tracking
# Monitors sequential task execution, captures all events and logs

set -e

# Configuration
NAMESPACE="${NAMESPACE:-agent-platform}"
REPOSITORY="${REPOSITORY:-5dlabs/cto-play-test}"
MONITOR_INTERVAL="${MONITOR_INTERVAL:-30}"  # seconds
MAX_RUNTIME="${MAX_RUNTIME:-7200}"  # 2 hours max

# Safely determine LOG_DIR - always create new directory for each monitoring session
if [[ -z "$LOG_DIR" ]]; then
    # Create base logs directory if it doesn't exist
    mkdir -p "workflow-monitor/logs" 2>/dev/null || true

    # Always create a new timestamped directory for clean session isolation
    # This prevents log mixing between different monitoring runs
    LOG_DIR="workflow-monitor/logs/$(date +%Y%m%d-%H%M%S)"

    # Create the log directory immediately to ensure logging functions work
    mkdir -p "$LOG_DIR" 2>/dev/null || true
    touch "$LOG_DIR/monitor.log" 2>/dev/null || true
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') ${BLUE}[INFO]${NC} $1" | tee -a "$LOG_DIR/monitor.log"
}

log_success() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') ${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_DIR/monitor.log"
}

log_warning() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') ${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_DIR/monitor.log"
}

log_error() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') ${RED}[ERROR]${NC} $1" | tee -a "$LOG_DIR/monitor.log"
}

log_debug() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') ${PURPLE}[DEBUG]${NC} $1" | tee -a "$LOG_DIR/monitor.log"
}

# Initialize log files
init_logs() {
    mkdir -p "$LOG_DIR"
    touch "$LOG_DIR/monitor.log"
    touch "$LOG_DIR/workflows.log"
    touch "$LOG_DIR/coderuns.log"
    touch "$LOG_DIR/pr-status.log"
    touch "$LOG_DIR/events.log"
    touch "$LOG_DIR/pod-logs.log"

    log_info "📊 Workflow Monitor Initialized"
    log_info "📁 Log Directory: $LOG_DIR"
    log_info "⏰ Monitor Interval: ${MONITOR_INTERVAL}s"
    log_info "⏱️  Max Runtime: ${MAX_RUNTIME}s"
    log_info "🏷️  Repository: $REPOSITORY"
    log_info "🔗 Namespace: $NAMESPACE"
    echo ""
}

# Capture system state
capture_system_state() {
    log_info "🔍 Capturing Initial System State..."

    # Workflows
    kubectl get workflows -n "$NAMESPACE" -o wide >> "$LOG_DIR/workflows.log" 2>&1

    # CodeRuns
    kubectl get coderun -n "$NAMESPACE" -o wide >> "$LOG_DIR/coderuns.log" 2>&1

    # PVCs
    kubectl get pvc -n "$NAMESPACE" >> "$LOG_DIR/pvcs.log" 2>&1

    # ConfigMaps
    kubectl get configmap -n "$NAMESPACE" >> "$LOG_DIR/configmaps.log" 2>&1

    # PR Status
    gh api repos/"$REPOSITORY"/pulls >> "$LOG_DIR/pr-status.log" 2>&1

    log_success "✅ Initial system state captured"
}

# Monitor workflows
monitor_workflows() {
    local last_workflow_count=0
    local last_coderun_count=0

    while true; do
        # Check workflows
        local workflow_count=$(kubectl get workflows -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)
        local workflow_status=$(kubectl get workflows -n "$NAMESPACE" -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,AGE:.status.startedAt --no-headers 2>/dev/null || echo "")

        if [[ $workflow_count -ne $last_workflow_count ]]; then
            log_info "🔄 Workflow count changed: $last_workflow_count → $workflow_count"
            echo "$(date '+%Y-%m-%d %H:%M:%S') - Workflows: $workflow_count" >> "$LOG_DIR/workflows.log"
            echo "$workflow_status" >> "$LOG_DIR/workflows.log"
            last_workflow_count=$workflow_count
        fi

        # Detailed workflow status
        kubectl get workflows -n "$NAMESPACE" -o wide >> "$LOG_DIR/workflows.log" 2>&1

        # Check CodeRuns
        local coderun_count=$(kubectl get coderun -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)
        local coderun_status=$(kubectl get coderun -n "$NAMESPACE" -o custom-columns=NAME:.metadata.name,TASK:.metadata.labels.task-id,SERVICE:.metadata.labels.service,MODEL:.spec.model,PHASE:.status.phase,AGE:.metadata.creationTimestamp --no-headers 2>/dev/null || echo "")

        if [[ $coderun_count -ne $last_coderun_count ]]; then
            log_info "🔄 CodeRun count changed: $last_coderun_count → $coderun_count"
            echo "$(date '+%Y-%m-%d %H:%M:%S') - CodeRuns: $coderun_count" >> "$LOG_DIR/coderuns.log"
            echo "$coderun_status" >> "$LOG_DIR/coderuns.log"
            last_coderun_count=$coderun_count
        fi

        # Detailed CodeRun status
        kubectl get coderun -n "$NAMESPACE" -o wide >> "$LOG_DIR/coderuns.log" 2>&1

        sleep "$MONITOR_INTERVAL"
    done
}

# Monitor PR status
monitor_prs() {
    while true; do
        # Get PR status
        local pr_data=$(gh api repos/"$REPOSITORY"/pulls 2>/dev/null || echo "[]")

        if [[ "$pr_data" != "[]" ]]; then
            echo "$(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG_DIR/pr-status.log"
            echo "$pr_data" | jq -r '.[] | "PR #\(.number): \(.title) - \(.state) - Created: \(.created_at)"' >> "$LOG_DIR/pr-status.log"
            echo "---" >> "$LOG_DIR/pr-status.log"
        fi

        sleep "$MONITOR_INTERVAL"
    done
}

# Monitor Kubernetes events
monitor_events() {
    while true; do
        # Get recent events
        kubectl get events -n "$NAMESPACE" --sort-by=.metadata.creationTimestamp | tail -20 >> "$LOG_DIR/events.log" 2>&1

        # Get workflow-specific events
        kubectl get workflows -n "$NAMESPACE" -o name 2>/dev/null | while read -r workflow; do
            kubectl describe "$workflow" -n "$NAMESPACE" 2>/dev/null | grep -A 20 "Events:" >> "$LOG_DIR/workflow-events.log" 2>&1 || true
        done

        # Capture pod logs for ALL CodeRuns (running, succeeded, failed)
        kubectl get coderun -n "$NAMESPACE" -o name 2>/dev/null | while read -r coderun; do
            # Extract pod name from CodeRun
            pod_name=$(kubectl get "$coderun" -n "$NAMESPACE" -o jsonpath='{.status.podName}' 2>/dev/null || echo "")
            if [[ -n "$pod_name" ]]; then
                # Check if we've already captured complete logs for this pod
                if ! grep -q "COMPLETE LOGS: $pod_name" "$LOG_DIR/pod-logs.log" 2>/dev/null; then
                    log_info "📝 Capturing complete logs from pod: $pod_name"

                    # Get CodeRun phase for context
                    coderun_phase=$(kubectl get "$coderun" -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")

                    {
                        echo "=== COMPLETE LOGS: $pod_name ==="
                        echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
                        echo "CodeRun: $coderun"
                        echo "Phase: $coderun_phase"
                        echo ""

                        # Try to get complete logs (not just tail)
                        if kubectl logs "$pod_name" -n "$NAMESPACE" --all-containers=true 2>/dev/null; then
                            echo ""
                            echo "✅ Successfully captured complete logs for $pod_name"
                        else
                            echo "❌ Failed to get logs from $pod_name (may be cleaned up)"
                        fi

                        echo ""
                        echo "=== END COMPLETE LOGS ==="
                        echo ""
                    } >> "$LOG_DIR/pod-logs.log"
                fi
            fi
        done

        # Also capture logs from any workflow-related pods (not just CodeRuns)
        kubectl get pods -n "$NAMESPACE" -l service=cto-play-test -o json 2>/dev/null | jq -r '.items[] | select(.status.phase != "Pending") | .metadata.name' | while read -r pod_name; do
            if [[ -n "$pod_name" ]] && ! grep -q "WORKFLOW POD: $pod_name" "$LOG_DIR/pod-logs.log" 2>/dev/null; then
                pod_phase=$(kubectl get pod "$pod_name" -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")

                log_debug "📝 Capturing logs from workflow pod: $pod_name ($pod_phase)"

                {
                    echo "=== WORKFLOW POD: $pod_name ==="
                    echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
                    echo "Phase: $pod_phase"
                    echo ""

                    if kubectl logs "$pod_name" -n "$NAMESPACE" --all-containers=true 2>/dev/null; then
                        echo ""
                        echo "✅ Successfully captured workflow pod logs for $pod_name"
                    else
                        echo "❌ Failed to get logs from workflow pod $pod_name"
                    fi

                    echo ""
                    echo "=== END WORKFLOW POD LOGS ==="
                    echo ""
                } >> "$LOG_DIR/pod-logs.log"
            fi
        done

        sleep "$((MONITOR_INTERVAL * 2))"  # Less frequent for events and logs
    done
}

# Generate summary report
generate_report() {
    log_info "📊 Generating Workflow Execution Report..."

    local report_file="$LOG_DIR/workflow-report.md"

    cat > "$report_file" << EOF
# Workflow Execution Report
Generated: $(date)

## Configuration
- **Repository**: $REPOSITORY
- **Namespace**: $NAMESPACE
- **Monitor Interval**: ${MONITOR_INTERVAL}s
- **Max Runtime**: ${MAX_RUNTIME}s
- **Log Directory**: $LOG_DIR

## Final State Summary

### Workflows
\`\`\`
$(kubectl get workflows -n "$NAMESPACE" -o wide 2>/dev/null || echo "No workflows found")
\`\`\`

### CodeRuns
\`\`\`
$(kubectl get coderun -n "$NAMESPACE" -o wide 2>/dev/null || echo "No CodeRuns found")
\`\`\`

### Pull Requests
\`\`\`
$(gh api repos/"$REPOSITORY"/pulls 2>/dev/null | jq -r '.[] | "PR #\(.number): \(.title) - \(.state)"' 2>/dev/null || echo "No PRs found")
\`\`\`

### Key Events Timeline
\`\`\`
$(grep -E "(Workflow|CodeRun|PR|ERROR|FAILED|SUCCESS)" "$LOG_DIR/monitor.log" | head -20)
\`\`\`

## Log Files
- **Main Log**: $LOG_DIR/monitor.log
- **Workflows**: $LOG_DIR/workflows.log
- **CodeRuns**: $LOG_DIR/coderuns.log
- **PR Status**: $LOG_DIR/pr-status.log
- **Events**: $LOG_DIR/events.log
- **PVCs**: $LOG_DIR/pvcs.log
- **ConfigMaps**: $LOG_DIR/configmaps.log

EOF

    log_success "✅ Report generated: $report_file"
}

# Cleanup function
cleanup() {
    log_info "🧹 Cleaning up monitoring processes..."
    kill $(jobs -p) 2>/dev/null || true
    generate_report
    log_info "✅ Monitoring completed"
    exit 0
}

# Signal handling
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    log_info "🚀 Starting Workflow Monitor..."
    echo "══════════════════════════════════════════════════════════"
    echo "🎯 WORKFLOW MONITOR - Sequential Execution Tracking"
    echo "══════════════════════════════════════════════════════════"
    echo ""

    # Initialize
    init_logs
    capture_system_state

    # Start monitoring processes in background
    monitor_workflows &
    WORKFLOW_PID=$!

    monitor_prs &
    PR_PID=$!

    monitor_events &
    EVENTS_PID=$!

    log_info "📈 Started monitoring processes:"
    log_info "  • Workflows: PID $WORKFLOW_PID"
    log_info "  • PR Status: PID $PR_PID"
    log_info "  • Events: PID $EVENTS_PID"

    # Wait for completion or timeout
    local start_time=$(date +%s)

    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        if [[ $elapsed -ge $MAX_RUNTIME ]]; then
            log_warning "⏰ Maximum runtime reached (${MAX_RUNTIME}s)"
            break
        fi

        # Check if any workflows are still active (Running OR Suspended)
        local active_workflows=$(kubectl get workflows -n "$NAMESPACE" -o json 2>/dev/null | jq -r '.items[] | select(.status.phase == "Running" or .status.phase == "Suspended") | .metadata.name' | wc -l)
        local running_coderuns=$(kubectl get coderun -n "$NAMESPACE" --field-selector status.phase=Running -o name 2>/dev/null | wc -l)

        # Specifically check for main play workflows (not resume workflows)
        local main_workflow_active=$(kubectl get workflows -n "$NAMESPACE" -l workflow-type=play-orchestration -o json 2>/dev/null | jq -r '.items[] | select(.status.phase == "Running" or .status.phase == "Suspended") | .metadata.name' | wc -l)

        if [[ $active_workflows -eq 0 && $running_coderuns -eq 0 ]]; then
            log_success "✅ No active workflows or CodeRuns detected"
            sleep 60  # Wait a bit to ensure everything is really done
            local final_active_workflows=$(kubectl get workflows -n "$NAMESPACE" -o json 2>/dev/null | jq -r '.items[] | select(.status.phase == "Running" or .status.phase == "Suspended") | .metadata.name' | wc -l)
            if [[ $final_active_workflows -eq 0 ]]; then
                log_success "🎉 All workflows completed!"
                break
            fi
        elif [[ $main_workflow_active -eq 0 ]]; then
            log_success "✅ Main workflow completed (only resume workflows remain)"
            sleep 30  # Brief wait before final check
            local final_main_workflow=$(kubectl get workflows -n "$NAMESPACE" -l workflow-type=play-orchestration -o json 2>/dev/null | jq -r '.items[] | select(.status.phase == "Running" or .status.phase == "Suspended") | .metadata.name' | wc -l)
            if [[ $final_main_workflow -eq 0 ]]; then
                log_success "🎉 Main workflow fully completed!"
                break
            fi
        fi

        log_debug "⏳ Monitoring... (${elapsed}s elapsed, ${active_workflows} active workflows, ${running_coderuns} CodeRuns running, main workflow: ${main_workflow_active})"
        sleep 60
    done

    # Generate final report
    generate_report

    log_success "🎯 Workflow Monitor completed successfully"
    echo ""
    echo "📊 Final Report: $LOG_DIR/workflow-report.md"
    echo "📁 All logs saved to: $LOG_DIR/"
}

# Run main function
main "$@"
