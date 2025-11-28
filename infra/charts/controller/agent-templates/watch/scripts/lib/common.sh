#!/bin/bash
# Common utilities for E2E Watch scripts
# Source this first in all scripts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
  echo -e "${BLUE}ℹ️  $*${NC}"
}

log_success() {
  echo -e "${GREEN}✅ $*${NC}"
}

log_warn() {
  echo -e "${YELLOW}⚠️  $*${NC}"
}

log_error() {
  echo -e "${RED}❌ $*${NC}" >&2
}

log_step() {
  echo -e "${BLUE}➡️  $*${NC}"
}

# Check required commands exist
require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" &> /dev/null; then
    log_error "Required command not found: $cmd"
    exit 1
  fi
}

# Check required environment variable
require_env() {
  local var="$1"
  if [ -z "${!var:-}" ]; then
    log_error "Required environment variable not set: $var"
    exit 1
  fi
}

# Retry a command with exponential backoff
# Usage: retry 5 30 some_command arg1 arg2
retry() {
  local max_attempts="$1"
  local delay="$2"
  shift 2
  local attempt=1

  while [ $attempt -le "$max_attempts" ]; do
    if "$@"; then
      return 0
    fi
    
    if [ $attempt -eq "$max_attempts" ]; then
      log_error "Command failed after $max_attempts attempts: $*"
      return 1
    fi
    
    log_warn "Attempt $attempt failed, retrying in ${delay}s..."
    sleep "$delay"
    delay=$((delay * 2))
    attempt=$((attempt + 1))
  done
}

# Poll until a condition is true
# Usage: poll_until 60 10 "description" check_function
poll_until() {
  local timeout="$1"
  local interval="$2"
  local description="$3"
  shift 3
  
  local elapsed=0
  log_info "Polling: $description (timeout: ${timeout}s, interval: ${interval}s)"
  
  while [ $elapsed -lt "$timeout" ]; do
    if "$@"; then
      log_success "$description - complete"
      return 0
    fi
    
    sleep "$interval"
    elapsed=$((elapsed + interval))
    echo -n "."
  done
  
  echo ""
  log_error "Timeout waiting for: $description"
  return 1
}

# Write JSON output
json_output() {
  local status="$1"
  local message="$2"
  shift 2
  
  echo "{"
  echo "  \"status\": \"$status\","
  echo "  \"message\": \"$message\""
  for pair in "$@"; do
    echo "  ,$pair"
  done
  echo "}"
}

# Parse key=value arguments
parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --*=*)
        local key="${1#--}"
        local var="${key%%=*}"
        local value="${key#*=}"
        # Convert dashes to underscores for variable names
        var="${var//-/_}"
        declare -g "$var=$value"
        ;;
      --*)
        local var="${1#--}"
        var="${var//-/_}"
        shift
        declare -g "$var=$1"
        ;;
    esac
    shift
  done
}

