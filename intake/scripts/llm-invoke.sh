#!/usr/bin/env bash
# Harness-agnostic LLM invocation adapter for intake workflows.
#
# Default behavior preserves the existing OpenClaw path when available.
# Set CTO_LLM_INVOKE_CMD to route the same argv through another harness, e.g.
#   CTO_LLM_INVOKE_CMD="hermes-llm-invoke"
#   CTO_LLM_INVOKE_CMD="lobster run --mode tool intake/workflows/llm.lobster.yaml"
set -euo pipefail

ROOT="${WORKSPACE:-$(cd "$(dirname "$0")/../.." && pwd)}"

if [ -n "${CTO_LLM_INVOKE_CMD:-}" ]; then
  # Split the configured command with normal shell parsing, then preserve the
  # caller's original argv as separate words. Passing PATH through explicitly is
  # important because some harnesses (including Lobster exec) sanitize PATH.
  exec env PATH="$PATH" bash -c 'cmd=( $CTO_LLM_INVOKE_CMD ); exec "${cmd[@]}" "$@"' _ "$@"
fi

if [ -n "${CTO_LLM_INVOKE_FALLBACK_OPENCLAW:-}" ] && command -v openclaw.invoke >/dev/null 2>&1; then
  exec "$ROOT/intake/scripts/openclaw-invoke-retry.sh" "$@"
fi

cat >&2 <<'EOF'
llm-invoke: No LLM harness command configured.
Set CTO_LLM_INVOKE_CMD to a command that accepts the OpenClaw-compatible
llm-task argv shape, for example:
  CTO_LLM_INVOKE_CMD="hermes-llm-invoke"
Expected argv shape:
  --tool llm-task --action json|text --args-json '{...}'
  --tool llm-task --action json|text --args-file /path/to/args.json
EOF
exit 127
