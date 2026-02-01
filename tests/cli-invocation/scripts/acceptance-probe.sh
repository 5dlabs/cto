#!/usr/bin/env bash
# =============================================================================
# Acceptance Criteria Probe - Mirrors Controller's acceptance-probe.sh.hbs
# =============================================================================
#
# Counts checkboxes in acceptance-criteria.md and determines pass/fail.
# 90% completion threshold required for "passed" status.
#
# Usage:
#   source /scripts/acceptance-probe.sh
#   probe_acceptance_criteria
#
# Output:
#   Writes "passed" or "failed" to /workspace/.acceptance_result
#
# =============================================================================

# Probe acceptance criteria and write result
probe_acceptance_criteria() {
  local criteria_file="${WORKSPACE:-/workspace}/task/acceptance-criteria.md"
  local result_file="${WORKSPACE:-/workspace}/.acceptance_result"
  local threshold="${ACCEPTANCE_THRESHOLD:-90}"
  
  echo "=== Acceptance Criteria Probe ===" >&2
  echo "  File: ${criteria_file}" >&2
  echo "  Threshold: ${threshold}%" >&2
  
  if [[ ! -f "${criteria_file}" ]]; then
    echo "❌ Acceptance criteria file not found: ${criteria_file}" >&2
    echo "failed" > "${result_file}"
    return 1
  fi
  
  # Count checkboxes
  local total_boxes=$(grep -c '\- \[[ x]\]' "${criteria_file}" 2>/dev/null || echo 0)
  local checked_boxes=$(grep -c '\- \[x\]' "${criteria_file}" 2>/dev/null || echo 0)
  
  if [[ ${total_boxes} -eq 0 ]]; then
    echo "⚠ No checkboxes found in acceptance criteria" >&2
    echo "failed" > "${result_file}"
    return 1
  fi
  
  # Calculate percentage
  local percentage=$((checked_boxes * 100 / total_boxes))
  
  echo "  Checked: ${checked_boxes}/${total_boxes} (${percentage}%)" >&2
  
  # Determine result
  if [[ ${percentage} -ge ${threshold} ]]; then
    echo "✅ PASSED - Acceptance criteria met (${percentage}% >= ${threshold}%)" >&2
    echo "passed" > "${result_file}"
    return 0
  else
    echo "❌ FAILED - Acceptance criteria not met (${percentage}% < ${threshold}%)" >&2
    echo "failed" > "${result_file}"
    return 1
  fi
}

# Display acceptance criteria status (for debugging)
show_acceptance_status() {
  local criteria_file="${WORKSPACE:-/workspace}/task/acceptance-criteria.md"
  
  if [[ ! -f "${criteria_file}" ]]; then
    echo "No acceptance criteria file found" >&2
    return 1
  fi
  
  echo "=== Acceptance Criteria Status ===" >&2
  grep '\- \[[ x]\]' "${criteria_file}" | while read -r line; do
    if [[ "${line}" == *"[x]"* ]]; then
      echo "  ✅ ${line#*] }" >&2
    else
      echo "  ⬜ ${line#*] }" >&2
    fi
  done
}

# Run probe if executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  probe_acceptance_criteria
  exit $?
fi
