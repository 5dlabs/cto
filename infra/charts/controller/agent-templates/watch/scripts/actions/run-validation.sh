#!/bin/bash
# Run local validation: cargo fmt, clippy, test
# Usage: run-validation.sh [--repo-dir /workspace/repo] [--fix]
#
# Exit: 0 if all pass, 1 if any fail
# With --fix: Automatically apply fmt fixes before checking

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/common.sh"

# Parse arguments
repo_dir="${REPO_DIR:-/workspace/repo}"
fix_mode="false"

while [ $# -gt 0 ]; do
  case "$1" in
    --repo-dir) repo_dir="$2"; shift 2 ;;
    --fix) fix_mode="true"; shift ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

cd "$repo_dir"

log_info "Running validation in $repo_dir"

# Step 1: Format
log_step "Running cargo fmt..."
if [ "$fix_mode" = "true" ]; then
  if ! cargo fmt --all; then
    log_error "cargo fmt failed"
    exit 1
  fi
  log_success "Formatting applied"
else
  if ! cargo fmt --all -- --check; then
    log_error "Formatting check failed - run with --fix to auto-fix"
    exit 1
  fi
  log_success "Formatting check passed"
fi

# Step 2: Clippy (pedantic per project rules)
log_step "Running cargo clippy (pedantic)..."
if ! cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic; then
  log_error "Clippy check failed"
  exit 1
fi
log_success "Clippy check passed"

# Step 3: Tests
log_step "Running cargo test..."
if ! cargo test --workspace --all-features; then
  log_error "Tests failed"
  exit 1
fi
log_success "All tests passed"

# Step 4: Check for YAML changes and lint them
log_step "Checking for YAML changes..."
yaml_files=$(git diff --name-only HEAD 2>/dev/null | grep -E '\.ya?ml$' || true)
if [ -n "$yaml_files" ]; then
  log_info "YAML files changed, running yamllint..."
  if command -v yamllint &> /dev/null; then
    if ! echo "$yaml_files" | xargs yamllint; then
      log_error "YAML lint failed"
      exit 1
    fi
    log_success "YAML lint passed"
  else
    log_warn "yamllint not installed, skipping YAML validation"
  fi
else
  log_info "No YAML changes to lint"
fi

log_success "All validation checks passed"

