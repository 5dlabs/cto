#!/bin/bash
# =============================================================================
# Unified Agent Entrypoint
# =============================================================================
#
# Reads agent identity from environment variables and configures the runtime
# accordingly. Every agent uses this same entrypoint; what differs is the
# environment injected by K8s (via cto-config.json or pod spec).
#
# Required env vars:
#   AGENT_NAME      - Agent identifier (e.g., "rex", "morgan", "blaze")
#
# Optional env vars:
#   AGENT_ROLE      - Role description (e.g., "rust-engineer", "pm")
#   AGENT_CLI       - Preferred coding CLI (default: "claude")
#   AGENT_MODEL     - Default LLM model
#   GITHUB_APP_NAME - GitHub App identity (e.g., "5DLabs-Rex")
#   CTO_CONFIG_PATH - Path to cto-config.json (default: /etc/cto/config.json)
#   LOBSTER_WORKFLOWS_DIR - Lobster workflow directory (default: /etc/cto/workflows)
#
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
AGENT_NAME="${AGENT_NAME:-unknown}"
AGENT_ROLE="${AGENT_ROLE:-agent}"
AGENT_CLI="${AGENT_CLI:-claude}"
AGENT_MODEL="${AGENT_MODEL:-}"
CTO_CONFIG_PATH="${CTO_CONFIG_PATH:-/etc/cto/config.json}"
LOBSTER_WORKFLOWS_DIR="${LOBSTER_WORKFLOWS_DIR:-/etc/cto/workflows}"

# ---------------------------------------------------------------------------
# Banner
# ---------------------------------------------------------------------------
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  CTO Platform — Unified Agent Image                        ║"
echo "╠══════════════════════════════════════════════════════════════╣"
printf "║  Agent:    %-48s ║\n" "$AGENT_NAME"
printf "║  Role:     %-48s ║\n" "$AGENT_ROLE"
printf "║  CLI:      %-48s ║\n" "$AGENT_CLI"
if [ -n "$AGENT_MODEL" ]; then
  printf "║  Model:    %-48s ║\n" "$AGENT_MODEL"
fi
if [ -n "${GITHUB_APP_NAME:-}" ]; then
  printf "║  GitHub:   %-48s ║\n" "$GITHUB_APP_NAME"
fi
echo "╚══════════════════════════════════════════════════════════════╝"

# ---------------------------------------------------------------------------
# GitHub App authentication (if app credentials are available)
# ---------------------------------------------------------------------------
if [ -n "${GITHUB_APP_ID:-}" ] && [ -n "${GITHUB_APP_PRIVATE_KEY:-}" ]; then
  echo "[agent-entrypoint] Authenticating GitHub App: ${GITHUB_APP_NAME:-unknown}..."
  if command -v github-app-auth >/dev/null 2>&1; then
    github-app-auth || echo "[agent-entrypoint] Warning: GitHub App auth failed"
  fi
fi

# ---------------------------------------------------------------------------
# Load cto-config.json if available (extract agent-specific settings)
# ---------------------------------------------------------------------------
if [ -f "$CTO_CONFIG_PATH" ]; then
  echo "[agent-entrypoint] Loading config from $CTO_CONFIG_PATH"

  # Extract agent-specific CLI preference if not overridden
  if [ "$AGENT_CLI" = "claude" ] && command -v jq >/dev/null 2>&1; then
    CONFIG_CLI=$(jq -r --arg name "$AGENT_NAME" '.agents[$name].cli // empty' "$CTO_CONFIG_PATH" 2>/dev/null || true)
    if [ -n "$CONFIG_CLI" ]; then
      AGENT_CLI="$CONFIG_CLI"
      echo "[agent-entrypoint] CLI from config: $AGENT_CLI"
    fi
  fi

  # Extract agent-specific model if not overridden
  if [ -z "$AGENT_MODEL" ] && command -v jq >/dev/null 2>&1; then
    CONFIG_MODEL=$(jq -r --arg name "$AGENT_NAME" '.agents[$name].model // empty' "$CTO_CONFIG_PATH" 2>/dev/null || true)
    if [ -n "$CONFIG_MODEL" ]; then
      AGENT_MODEL="$CONFIG_MODEL"
      echo "[agent-entrypoint] Model from config: $AGENT_MODEL"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Export resolved identity for downstream tools
# ---------------------------------------------------------------------------
export AGENT_NAME AGENT_ROLE AGENT_CLI AGENT_MODEL
export CTO_CONFIG_PATH LOBSTER_WORKFLOWS_DIR

# ---------------------------------------------------------------------------
# Verify the selected CLI is available
# ---------------------------------------------------------------------------
verify_cli() {
  local cli="$1"
  case "$cli" in
    claude)       command -v claude >/dev/null 2>&1 ;;
    cursor)       command -v cursor-agent >/dev/null 2>&1 ;;
    codex)        command -v codex >/dev/null 2>&1 ;;
    factory)      command -v droid >/dev/null 2>&1 ;;
    opencode)     command -v opencode >/dev/null 2>&1 ;;
    gemini)       command -v gemini >/dev/null 2>&1 ;;
    code)         command -v code >/dev/null 2>&1 ;;
    openclaw)     command -v openclaw >/dev/null 2>&1 ;;
    *)            echo "[agent-entrypoint] Warning: Unknown CLI '$cli'"; return 1 ;;
  esac
}

if ! verify_cli "$AGENT_CLI"; then
  echo "[agent-entrypoint] Warning: CLI '$AGENT_CLI' not found, falling back to claude"
  AGENT_CLI="claude"
  export AGENT_CLI
fi

echo "[agent-entrypoint] Agent '$AGENT_NAME' ready (cli=$AGENT_CLI)"

# ---------------------------------------------------------------------------
# Delegate to openclaw or run the provided command
# ---------------------------------------------------------------------------
if [ $# -eq 0 ] || [ "$1" = "--help" ]; then
  exec openclaw "$@"
else
  exec "$@"
fi
