#!/usr/bin/env bash
# =============================================================================
# ralph-response-analyzer.sh - Response Analysis Library for Dual Ralph System
# =============================================================================
# Provides functions for analyzing AI agent outputs to detect completion,
# EXIT_SIGNAL, struggle patterns, and other response characteristics.
#
# Source this file from other Ralph scripts:
#   source "${SCRIPT_DIR}/ralph-response-analyzer.sh"
# =============================================================================

# Default configuration (can be overridden before sourcing)
: "${RA_COMPLETION_THRESHOLD:=2}"
: "${RA_REQUIRE_EXIT_SIGNAL:=true}"

# =============================================================================
# EXIT_SIGNAL Detection
# =============================================================================

# Extract EXIT_SIGNAL from agent output
# Looks for EXIT_SIGNAL: true/false in RALPH_STATUS block or standalone
# Args: $1 = output text
# Returns: "true", "false", or "missing"
ra_extract_exit_signal() {
  local output="$1"
  
  # Look for RALPH_STATUS block first
  if echo "$output" | grep -qE 'RALPH_STATUS'; then
    # Extract from block
    if echo "$output" | grep -qiE 'EXIT_SIGNAL[:\s]*true'; then
      echo "true"
      return
    elif echo "$output" | grep -qiE 'EXIT_SIGNAL[:\s]*false'; then
      echo "false"
      return
    fi
  fi
  
  # Fallback to standalone detection
  if echo "$output" | grep -qiE 'EXIT_SIGNAL:\s*true'; then
    echo "true"
  elif echo "$output" | grep -qiE 'EXIT_SIGNAL:\s*false'; then
    echo "false"
  else
    echo "missing"
  fi
}

# =============================================================================
# Completion Indicators Detection
# =============================================================================

# Count completion indicators in agent output (heuristic detection)
# Args: $1 = output text
# Returns: number of matched patterns
ra_count_completion_indicators() {
  local output="$1"
  local count=0
  
  # Completion patterns - higher confidence patterns
  local high_confidence_patterns=(
    "all tasks? (are |have been )?complete"
    "project (is )?complete"
    "all requirements (are |have been )?met"
    "all gates? pass(ed|ing)?"
    "STATUS:\s*COMPLETE"
    "COMPLETE:\s*true"
    "work (is )?done"
    "nothing (left |more )?to (do|implement)"
  )
  
  # Medium confidence patterns
  local medium_confidence_patterns=(
    "remediation (is )?complete"
    "fix(es)? applied successfully"
    "issue(s)? resolved"
    "phase complete"
    "implementation complete"
    "successfully (fixed|resolved|completed)"
  )
  
  # Low confidence patterns (less weight)
  local low_confidence_patterns=(
    "done"
    "finished"
    "complete"
  )
  
  # High confidence = 2 points each
  for pattern in "${high_confidence_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      count=$((count + 2))
    fi
  done
  
  # Medium confidence = 1 point each
  for pattern in "${medium_confidence_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      count=$((count + 1))
    fi
  done
  
  # Low confidence = 0.5 points but we use integers, so only if multiple
  local low_matches=0
  for pattern in "${low_confidence_patterns[@]}"; do
    if echo "$output" | grep -qiE "\\b${pattern}\\b"; then
      low_matches=$((low_matches + 1))
    fi
  done
  # Only count low confidence if 2+ matches
  if [[ $low_matches -ge 2 ]]; then
    count=$((count + 1))
  fi
  
  echo "$count"
}

# =============================================================================
# Dual-Condition Exit Check
# =============================================================================

# Determine if should exit based on dual-condition check
# Args: $1 = output text, $2 = threshold (optional), $3 = require_signal (optional)
# Returns: "exit" or "continue"
ra_should_exit_gracefully() {
  local output="$1"
  local threshold="${2:-$RA_COMPLETION_THRESHOLD}"
  local require_signal="${3:-$RA_REQUIRE_EXIT_SIGNAL}"
  
  local exit_signal
  local completion_indicators
  
  exit_signal=$(ra_extract_exit_signal "$output")
  completion_indicators=$(ra_count_completion_indicators "$output")
  
  # Decision matrix:
  # - indicators >= threshold AND EXIT_SIGNAL=true -> Exit
  # - indicators >= threshold AND EXIT_SIGNAL=false -> Continue
  # - indicators >= threshold AND missing -> Continue (default to false)
  # - indicators < threshold -> Continue regardless of signal
  
  if [[ "$require_signal" == "true" ]]; then
    if [[ "$exit_signal" == "true" ]] && [[ "$completion_indicators" -ge "$threshold" ]]; then
      echo "exit"
      return
    fi
    
    if [[ "$exit_signal" == "false" ]]; then
      echo "continue"
      return
    fi
    
    # Missing signal with high indicators - conservative approach, continue
    if [[ "$completion_indicators" -ge "$threshold" ]] && [[ "$exit_signal" == "missing" ]]; then
      echo "continue"
      return
    fi
  else
    # Legacy mode - just use completion indicators
    if [[ "$completion_indicators" -ge "$threshold" ]]; then
      echo "exit"
      return
    fi
  fi
  
  echo "continue"
}

# =============================================================================
# Struggle Detection
# =============================================================================

# Detect if agent is struggling (going in circles, stuck, confused)
# Args: $1 = output text
# Returns: 0 if struggling, 1 if not
ra_detect_struggle() {
  local output="$1"
  
  # Strong struggle indicators
  local strong_patterns=(
    "I('m| am) (completely )?stuck"
    "I('m| am) unable to (proceed|continue|fix)"
    "I cannot (figure out|determine|solve|understand)"
    "This (is|seems) (impossible|unfixable)"
    "I need (human |manual )?(help|intervention|assistance)"
    "I('m| am) going in circles"
    "I('ve| have) tried everything"
    "I don't know (what|how) to"
    "beyond my (ability|capabilities)"
  )
  
  # Repetition patterns (indicates looping)
  local repetition_patterns=(
    "ERROR:.*ERROR:.*ERROR:"
    "failed.*failed.*failed"
    "trying again.*trying again"
    "same (error|issue|problem).*again"
  )
  
  # Confusion patterns
  local confusion_patterns=(
    "I('m| am) confused"
    "I don't understand"
    "this doesn't make sense"
    "contradictory (requirements|instructions)"
    "unclear (what|how|why)"
  )
  
  for pattern in "${strong_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      return 0
    fi
  done
  
  for pattern in "${repetition_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      return 0
    fi
  done
  
  for pattern in "${confusion_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      return 0
    fi
  done
  
  return 1
}

# =============================================================================
# Progress Detection
# =============================================================================

# Detect if agent is making progress
# Args: $1 = output text
# Returns: 0 if progress detected, 1 if not
ra_detect_progress() {
  local output="$1"
  
  local progress_patterns=(
    "created? (file|directory|function|class|module)"
    "(modified|updated|changed|edited) (file|code|config)"
    "commit(ted)?"
    "test(s)? pass(ed|ing)"
    "build (succeeded|successful)"
    "fixed (the |a )?(bug|issue|problem|error)"
    "implemented"
    "added (new )?(feature|functionality|support)"
    "refactored"
    "resolved"
  )
  
  for pattern in "${progress_patterns[@]}"; do
    if echo "$output" | grep -qiE "$pattern"; then
      return 0
    fi
  done
  
  return 1
}

# =============================================================================
# Error Detection
# =============================================================================

# Extract errors from agent output
# Args: $1 = output text
# Returns: extracted error messages (one per line)
ra_extract_errors() {
  local output="$1"
  
  # Common error patterns
  echo "$output" | grep -iE '(error|exception|failed|failure|fatal)' | head -20
}

# Count unique errors in output
# Args: $1 = output text
# Returns: count of unique error lines
ra_count_errors() {
  local output="$1"
  
  ra_extract_errors "$output" | sort -u | wc -l | tr -d ' '
}

# =============================================================================
# Response Summary
# =============================================================================

# Generate a summary of the response analysis
# Args: $1 = output text
# Returns: JSON summary
ra_analyze_response() {
  local output="$1"
  
  local exit_signal
  local completion_count
  local error_count
  local is_struggling
  local has_progress
  local exit_decision
  
  exit_signal=$(ra_extract_exit_signal "$output")
  completion_count=$(ra_count_completion_indicators "$output")
  error_count=$(ra_count_errors "$output")
  exit_decision=$(ra_should_exit_gracefully "$output")
  
  if ra_detect_struggle "$output"; then
    is_struggling="true"
  else
    is_struggling="false"
  fi
  
  if ra_detect_progress "$output"; then
    has_progress="true"
  else
    has_progress="false"
  fi
  
  jq -n \
    --arg exit_signal "$exit_signal" \
    --argjson completion_count "$completion_count" \
    --argjson error_count "$error_count" \
    --argjson is_struggling "$is_struggling" \
    --argjson has_progress "$has_progress" \
    --arg exit_decision "$exit_decision" \
    '{
      exitSignal: $exit_signal,
      completionIndicators: $completion_count,
      errorCount: $error_count,
      isStruggling: $is_struggling,
      hasProgress: $has_progress,
      exitDecision: $exit_decision,
      timestamp: (now | todate)
    }'
}

# =============================================================================
# Utility Functions
# =============================================================================

# Log function (if not already defined)
if ! declare -f ra_log >/dev/null 2>&1; then
  ra_log() { echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [ANALYZER] $*" >&2; }
fi

# Print analysis to stderr for debugging
ra_debug_analysis() {
  local output="$1"
  
  ra_log "=== Response Analysis ==="
  ra_log "EXIT_SIGNAL: $(ra_extract_exit_signal "$output")"
  ra_log "Completion indicators: $(ra_count_completion_indicators "$output")"
  ra_log "Error count: $(ra_count_errors "$output")"
  ra_log "Is struggling: $(ra_detect_struggle "$output" && echo "yes" || echo "no")"
  ra_log "Has progress: $(ra_detect_progress "$output" && echo "yes" || echo "no")"
  ra_log "Exit decision: $(ra_should_exit_gracefully "$output")"
  ra_log "========================="
}
