#!/usr/bin/env bash
# Harness-agnostic LLM invocation adapter for intake workflows.
#
# Lobster workflows call this stable OpenClaw-compatible llm-task interface.
# Backend selection is centralized here so the workflow can run through a
# universal ACP/ACPX path, an OpenClaw gateway, or an explicit compatibility
# adapter without changing workflow YAML.
set -euo pipefail

ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"
BACKEND="${CTO_LLM_INVOKE_BACKEND:-auto}"
ACPX_ADAPTER="$ROOT/intake/scripts/acpx-llm-task.py"
DIRECT_ADAPTER="$ROOT/intake/scripts/real-llm-invoke.py"

run_configured_cmd() {
  # Split the configured command with normal shell parsing, then preserve the
  # caller's original argv as separate words. Passing PATH through explicitly is
  # important because some harnesses (including Lobster exec) sanitize PATH.
  exec env PATH="$PATH" bash -c 'cmd=( $CTO_LLM_INVOKE_CMD ); exec "${cmd[@]}" "$@"' _ "$@"
}

run_acp() {
  if [ -x "$ACPX_ADAPTER" ]; then
    exec "$ACPX_ADAPTER" "$@"
  fi
  cat >&2 <<EOF
llm-invoke: CTO_LLM_INVOKE_BACKEND=acp requested, but ACP adapter is missing or not executable:
  $ACPX_ADAPTER
Set CTO_LLM_INVOKE_CMD to an explicit llm-task command, or install the ACP adapter.
EOF
  exit 127
}

run_openclaw() {
  if command -v openclaw.invoke >/dev/null 2>&1; then
    exec "$ROOT/intake/scripts/openclaw-invoke-retry.sh" "$@"
  fi
  cat >&2 <<'EOF'
llm-invoke: OpenClaw backend requested, but openclaw.invoke is not on PATH.
Set CTO_LLM_INVOKE_CMD to an explicit llm-task command or use CTO_LLM_INVOKE_BACKEND=acp.
EOF
  exit 127
}

run_direct() {
  if [ -x "$DIRECT_ADAPTER" ]; then
    exec "$DIRECT_ADAPTER" "$@"
  fi
  cat >&2 <<EOF
llm-invoke: Direct backend requested, but direct adapter is missing or not executable:
  $DIRECT_ADAPTER
Set CTO_LLM_INVOKE_CMD to an explicit llm-task command or use CTO_LLM_INVOKE_BACKEND=acp.
EOF
  exit 127
}

if [ -n "${CTO_LLM_INVOKE_CMD:-}" ]; then
  run_configured_cmd "$@"
fi

case "$BACKEND" in
  acp|acpx)
    run_acp "$@"
    ;;
  openclaw)
    run_openclaw "$@"
    ;;
  direct|real)
    run_direct "$@"
    ;;
  auto|"")
    if command -v acpx >/dev/null 2>&1 && [ -x "$ACPX_ADAPTER" ]; then
      exec "$ACPX_ADAPTER" "$@"
    fi
    if [ -n "${CTO_LLM_INVOKE_FALLBACK_OPENCLAW:-}" ] && command -v openclaw.invoke >/dev/null 2>&1; then
      exec "$ROOT/intake/scripts/openclaw-invoke-retry.sh" "$@"
    fi
    ;;
  *)
    cat >&2 <<EOF
llm-invoke: Unknown CTO_LLM_INVOKE_BACKEND='$BACKEND'.
Supported values: auto, acp, openclaw, direct.
EOF
    exit 2
    ;;
esac

cat >&2 <<EOF
llm-invoke: No LLM harness backend is available.
Set one of:
  CTO_LLM_INVOKE_BACKEND=acp       # preferred universal ACP/ACPX backend
  CTO_LLM_INVOKE_BACKEND=openclaw  # OpenClaw gateway compatibility backend
  CTO_LLM_INVOKE_BACKEND=direct    # direct provider adapter
  CTO_LLM_INVOKE_CMD="/path/to/llm-task-adapter"
Expected argv shape:
  --tool llm-task --action json|text --args-json '{...}'
  --tool llm-task --action json|text --args-file /path/to/args.json
EOF
exit 127
