#!/bin/bash
# Poll GitHub Actions and get detailed failure reports
# Usage: poll-actions.sh --branch fix/my-branch [--pr-number 123] [--timeout 1800]
#
# This script:
# 1. Waits for all workflow runs to complete
# 2. Identifies failed runs
# 3. Downloads failed job logs
# 4. Outputs a structured failure report
#
# Output: JSON report with failed jobs and log excerpts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"

# Parse arguments
branch=""
pr_number=""
repo="${GITHUB_REPO:-5dlabs/cto}"
timeout="1800"  # 30 minutes default
log_lines="200"  # Lines of log to capture per failed job

while [ $# -gt 0 ]; do
  case "$1" in
    --branch) branch="$2"; shift 2 ;;
    --pr-number) pr_number="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --timeout) timeout="$2"; shift 2 ;;
    --log-lines) log_lines="$2"; shift 2 ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [ -z "$branch" ] && [ -z "$pr_number" ]; then
  log_error "Required: --branch or --pr-number"
  exit 1
fi

# If we have PR number but no branch, get the branch
if [ -z "$branch" ] && [ -n "$pr_number" ]; then
  branch=$(gh pr view "$pr_number" --repo "$repo" --json headRefName -q '.headRefName')
  log_info "Got branch from PR #$pr_number: $branch"
fi

log_info "Polling GitHub Actions for branch: $branch"

# ============================================================================
# Step 1: Wait for all runs to complete
# ============================================================================
log_step "Waiting for workflow runs to complete..."

start_time=$(date +%s)
while true; do
  current_time=$(date +%s)
  elapsed=$((current_time - start_time))
  
  if [ $elapsed -gt "$timeout" ]; then
    log_error "Timeout waiting for runs to complete"
    exit 1
  fi
  
  # Get all runs for this branch
  runs=$(gh run list --branch "$branch" --repo "$repo" --json databaseId,status,conclusion,name,workflowName 2>/dev/null || echo "[]")
  
  # Check if any are still in progress
  in_progress=$(echo "$runs" | jq '[.[] | select(.status == "in_progress" or .status == "queued")] | length')
  
  if [ "$in_progress" = "0" ]; then
    log_success "All workflow runs completed"
    break
  fi
  
  log_info "Waiting... ($in_progress runs still in progress, ${elapsed}s elapsed)"
  sleep 15
done

# ============================================================================
# Step 2: Identify failed runs
# ============================================================================
log_step "Checking for failed runs..."

runs=$(gh run list --branch "$branch" --repo "$repo" --json databaseId,status,conclusion,name,workflowName)
failed_runs=$(echo "$runs" | jq '[.[] | select(.conclusion == "failure")]')
failed_count=$(echo "$failed_runs" | jq 'length')

if [ "$failed_count" = "0" ]; then
  log_success "All runs passed!"
  echo '{"success": true, "failed_count": 0, "failures": []}'
  exit 0
fi

log_warn "Found $failed_count failed run(s)"

# ============================================================================
# Step 3: Get detailed failure info and logs for each failed run
# ============================================================================
log_step "Collecting failure details and logs..."

failures=()

for run_id in $(echo "$failed_runs" | jq -r '.[].databaseId'); do
  log_info "Analyzing run $run_id..."
  
  # Get run details including failed jobs
  run_info=$(gh run view "$run_id" --repo "$repo" --json jobs,name,workflowName,conclusion,url)
  
  workflow_name=$(echo "$run_info" | jq -r '.workflowName')
  run_name=$(echo "$run_info" | jq -r '.name')
  run_url=$(echo "$run_info" | jq -r '.url')
  
  # Get failed jobs
  failed_jobs=$(echo "$run_info" | jq '[.jobs[] | select(.conclusion == "failure")]')
  
  for job_idx in $(echo "$failed_jobs" | jq -r 'keys[]'); do
    job=$(echo "$failed_jobs" | jq ".[$job_idx]")
    job_name=$(echo "$job" | jq -r '.name')
    job_id=$(echo "$job" | jq -r '.databaseId')
    
    log_info "  Failed job: $job_name"
    
    # Get failed steps
    failed_steps=$(echo "$job" | jq '[.steps[] | select(.conclusion == "failure")]')
    
    # Get log for this specific job
    log_content=""
    if log_output=$(gh run view "$run_id" --repo "$repo" --log-failed 2>/dev/null); then
      # Extract just the last N lines relevant to this job
      log_content=$(echo "$log_output" | grep -A "$log_lines" "$job_name" | tail -"$log_lines" || echo "$log_output" | tail -"$log_lines")
    fi
    
    # Build failure entry
    failure_entry=$(jq -n \
      --arg workflow "$workflow_name" \
      --arg run_name "$run_name" \
      --arg run_id "$run_id" \
      --arg run_url "$run_url" \
      --arg job_name "$job_name" \
      --arg job_id "$job_id" \
      --argjson failed_steps "$failed_steps" \
      --arg log_excerpt "$log_content" \
      '{
        workflow: $workflow,
        run_name: $run_name,
        run_id: $run_id,
        run_url: $run_url,
        job_name: $job_name,
        job_id: $job_id,
        failed_steps: $failed_steps,
        log_excerpt: $log_excerpt
      }')
    
    failures+=("$failure_entry")
  done
done

# ============================================================================
# Step 4: Output structured report
# ============================================================================
log_step "Generating failure report..."

# Combine all failures into JSON array
failures_json="["
first=true
for f in "${failures[@]}"; do
  if [ "$first" = true ]; then
    first=false
  else
    failures_json+=","
  fi
  failures_json+="$f"
done
failures_json+="]"

# Create final report
report=$(jq -n \
  --argjson failed_count "$failed_count" \
  --argjson failures "$failures_json" \
  '{
    success: false,
    failed_count: $failed_count,
    failures: $failures,
    summary: ($failures | map("\(.workflow) / \(.job_name)") | join(", "))
  }')

echo "$report" | jq '.'

# Also save to file if WATCH_WORKSPACE is set
if [ -n "${WATCH_WORKSPACE:-}" ]; then
  echo "$report" > "$WATCH_WORKSPACE/action-failures.json"
  log_info "Failure report saved to $WATCH_WORKSPACE/action-failures.json"
fi

log_error "CI failures detected - see report above"
exit 1

