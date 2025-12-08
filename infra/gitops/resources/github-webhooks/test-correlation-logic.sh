#!/bin/bash

# Test script for GitHub webhook correlation logic
# This validates the task ID extraction and workflow targeting logic

set -e

echo "=== GitHub Webhook Correlation Logic Test Suite ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to simulate Go template extraction logic
extract_task_id() {
    local payload="$1"
    local method="$2"
    
    if [ "$method" = "label" ]; then
        # Extract from PR labels
        echo "$payload" | jq -r '.pull_request.labels[]?.name | select(startswith("task-")) | split("-")[1]' | head -1
    elif [ "$method" = "branch" ]; then
        # Extract from branch name
        local branch=$(echo "$payload" | jq -r '.pull_request.head.ref')
        if [[ "$branch" =~ ^task-([0-9]+) ]]; then
            echo "${BASH_REMATCH[1]}"
        elif [[ "$branch" =~ ^feature/task-([0-9]+) ]]; then
            echo "${BASH_REMATCH[1]}"
        fi
    fi
}

# Function to run a test case
run_test() {
    local test_name="$1"
    local payload="$2"
    local expected_id="$3"
    local expected_workflow="$4"
    
    echo -n "Testing: $test_name ... "
    
    # Try label extraction first
    local task_id=$(extract_task_id "$payload" "label")
    
    # Fallback to branch extraction if needed
    if [ -z "$task_id" ]; then
        task_id=$(extract_task_id "$payload" "branch")
    fi
    
    # Construct workflow name
    local workflow_name=""
    if [ -n "$task_id" ]; then
        workflow_name="play-task-${task_id}-workflow"
    else
        workflow_name="play-task-unknown-workflow"
    fi
    
    # Validate results
    if [ "$task_id" = "$expected_id" ] && [ "$workflow_name" = "$expected_workflow" ]; then
        echo -e "${GREEN}PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Expected: ID=$expected_id, Workflow=$expected_workflow"
        echo "  Got:      ID=$task_id, Workflow=$workflow_name"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test Case 1: Standard PR with task label
echo "Running Test Suite..."
echo "===================="
echo ""

payload1='{
  "pull_request": {
    "labels": [{"name": "task-5"}],
    "head": {"ref": "task-5-webhook-correlation"}
  }
}'
run_test "Standard PR Creation" "$payload1" "5" "play-task-5-workflow"

# Test Case 2: Multiple labels
payload2='{
  "pull_request": {
    "labels": [
      {"name": "enhancement"},
      {"name": "task-8"},
      {"name": "priority-high"}
    ],
    "head": {"ref": "feature/task-8-multi-agent"}
  }
}'
run_test "Multiple Labels" "$payload2" "8" "play-task-8-workflow"

# Test Case 3: Branch name fallback
payload3='{
  "pull_request": {
    "labels": [],
    "head": {"ref": "task-12-implement-feature"}
  }
}'
run_test "Branch Name Fallback" "$payload3" "12" "play-task-12-workflow"

# Test Case 4: Malformed label with branch fallback
payload4='{
  "pull_request": {
    "labels": [{"name": "task-abc"}],
    "head": {"ref": "task-15-valid-branch"}
  }
}'
run_test "Malformed Label Fallback" "$payload4" "15" "play-task-15-workflow"

# Test Case 5: Feature branch format
payload5='{
  "pull_request": {
    "labels": [],
    "head": {"ref": "feature/task-20-enhancement"}
  }
}'
run_test "Feature Branch Format" "$payload5" "20" "play-task-20-workflow"

# Test Case 6: No task identification
payload6='{
  "pull_request": {
    "labels": [{"name": "bug"}, {"name": "documentation"}],
    "head": {"ref": "fix/random-bug-fix"}
  }
}'
run_test "No Task Identification" "$payload6" "" "play-task-unknown-workflow"

# Test Case 7: Multiple task labels (use first)
payload7='{
  "pull_request": {
    "labels": [{"name": "task-1"}, {"name": "task-2"}],
    "head": {"ref": "multi-task-branch"}
  }
}'
run_test "Multiple Task Labels" "$payload7" "1" "play-task-1-workflow"

echo ""
echo "===================="
echo "Test Results Summary"
echo "===================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please review the correlation logic.${NC}"
    exit 1
fi