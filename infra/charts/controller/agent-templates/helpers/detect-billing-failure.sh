#!/bin/bash
# Billing Failure Detection and PR Alert Script
# 
# Usage: detect_billing_failure "$CLI_OUTPUT" "$EXIT_CODE" "$PR_NUMBER" "$REPO" "$AGENT_NAME"
#
# Detects billing/quota failures from CLI output and posts alert comment to PR

detect_billing_failure() {
  local cli_output="$1"
  local exit_code="$2"
  local pr_number="$3"
  local repo="$4"
  local agent_name="$5"
  local task_id="${6:-${TASK_ID:-unknown}}"
  local workflow_stage="${7:-${WORKFLOW_STAGE:-unknown}}"
  local workflow_name="${8:-${WORKFLOW_NAME:-unknown}}"
  local max_chars="${BILLING_FAILURE_LOG_MAX_CHARS:-4000}"
  
  # Skip if command succeeded
  [[ "$exit_code" == "0" ]] && return 0
  
  # Check for billing-related errors (case-insensitive)
  local billing_patterns=(
    "insufficient.*credit"
    "quota.*exceeded"
    "billing.*issue"
    "payment.*required"
    "rate.*limit.*exceeded"
    "usage.*limit"
    "account.*suspended"
    "subscription.*expired"
    "balance.*insufficient"
    "credit.*depleted"
  )
  
  local is_billing_error=false
  local matched_pattern=""
  
  for pattern in "${billing_patterns[@]}"; do
    if echo "$cli_output" | grep -iE "$pattern" >/dev/null 2>&1; then
      is_billing_error=true
      matched_pattern="$pattern"
      break
    fi
  done
  
  # If billing error detected, post alert comment in remediation format
  if [[ "$is_billing_error" == "true" ]]; then
    echo "🚨 BILLING FAILURE DETECTED: ${matched_pattern}"
    
    local trimmed_output="$cli_output"
    local cli_len=${#trimmed_output}
    if [[ "$cli_len" -gt "$max_chars" ]]; then
      trimmed_output="$(printf '%s' "$trimmed_output" | tail -c "$max_chars")"
      trimmed_output="(truncated last ${max_chars} characters)\n${trimmed_output}"
    fi
    
    local comment_body="🔴 Required Changes
**Issue Type**: [Regression]
**Severity**: [Critical]

### Description
${agent_name} could not proceed during the **${workflow_stage:-unknown}** stage for task ${task_id} (workflow ${workflow_name}) because the CLI reported a billing/quota failure that matched \`${matched_pattern}\`. This blocks the automation from completing.

### Acceptance Criteria Not Met
- [ ] Restore the provider's billing/quota so ${agent_name} can run end-to-end
- [ ] Re-run the multi-agent workflow for task ${task_id} after quota is restored
- [ ] Confirm ${agent_name} completes without provider billing errors

### Steps to Reproduce
1. Re-run the workflow for task ${task_id}
2. Observe the ${agent_name} stage exit with the billing/quota error shown below

### Error Details
~~~
${trimmed_output}
~~~

---
*Automatically posted by the agent platform to trigger remediation*"
    
    if [[ -n "$pr_number" && -n "$repo" ]]; then
      if command -v gh >/dev/null 2>&1 && [[ -n "$GITHUB_TOKEN" ]]; then
        if echo "$comment_body" | gh pr comment "$pr_number" \
          --repo "$repo" \
          --body-file - 2>/dev/null; then
          echo "✅ Alert comment posted to PR #${pr_number}"
        else
          echo "⚠️  Failed to post billing alert comment"
          return 2
        fi
      else
        echo "⚠️  GitHub CLI not available or GH token missing; cannot post billing alert"
        echo "$comment_body"
      fi
    else
      echo "⚠️  PR number or repository not set; printing billing alert locally"
      echo "$comment_body"
    fi
    
    return 1
  fi
  
  return 0
}

# Export for use in container scripts
export -f detect_billing_failure 2>/dev/null || true

