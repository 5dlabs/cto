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
  
  # If billing error detected, post alert comment
  if [[ "$is_billing_error" == "true" ]]; then
    echo "🚨 BILLING FAILURE DETECTED: ${matched_pattern}"
    echo "   Posting alert comment to PR #${pr_number}..."
    
    local comment_body="## 🚨 Billing/Quota Failure Alert

**Agent**: ${agent_name}  
**Error Type**: Billing/quota issue detected

### Error Details
\`\`\`
${cli_output}
\`\`\`

### Action Required
This agent failed due to billing or quota issues. Please:
1. Check billing account status for the CLI provider
2. Verify API quota limits
3. Ensure sufficient credits/balance
4. Restart workflow after resolving billing issues

### Detected Pattern
Matched: \`${matched_pattern}\`

---
*This is an automated alert from the agent platform.*"
    
    # Post comment using gh CLI
    if command -v gh >/dev/null 2>&1 && [[ -n "$GITHUB_TOKEN" ]]; then
      echo "$comment_body" | gh pr comment "$pr_number" \
        --repo "$repo" \
        --body-file - 2>/dev/null || {
        echo "⚠️  Failed to post comment (continuing anyway)"
      }
      echo "✅ Alert comment posted to PR #${pr_number}"
    else
      echo "⚠️  GitHub CLI not available or no token, cannot post comment"
    fi
    
    return 1
  fi
  
  return 0
}

# Export for use in container scripts
export -f detect_billing_failure 2>/dev/null || true

