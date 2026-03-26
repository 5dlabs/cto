#!/usr/bin/env bash
# Verify intake gateway runtime: Node ≥22, OpenClaw + standalone lobster on PATH.
# Per https://docs.openclaw.ai/tools/lobster — lobster should live on the same host as the gateway.
# Install / update: npm install -g openclaw@latest && npm install -g @clawdbot/lobster@latest
set -euo pipefail

code=0
echo "=== Intake gateway toolchain ==="
if command -v node >/dev/null 2>&1; then
  echo "node: $(node -v)"
  major="$(node -p 'parseInt(process.versions.node.split(".")[0],10)')"
  if [ "${major}" -lt 22 ] 2>/dev/null; then
    echo "WARN: OpenClaw targets Node >= 22 on current trees (have ${major})" >&2
    code=1
  fi
else
  echo "ERROR: node not on PATH" >&2
  exit 1
fi

if command -v openclaw >/dev/null 2>&1; then
  echo "openclaw: $(openclaw --version 2>&1 | head -1)"
else
  echo "ERROR: openclaw not on PATH" >&2
  exit 1
fi

if command -v lobster >/dev/null 2>&1; then
  echo "lobster: $(lobster --version 2>&1 | head -1)"
else
  echo "ERROR: lobster not on PATH (install: npm install -g @clawdbot/lobster@latest)" >&2
  exit 1
fi

if command -v npm >/dev/null 2>&1; then
  echo "npm @clawdbot/lobster published latest: $(npm view @clawdbot/lobster version 2>/dev/null || echo '?')"
fi

echo ""
echo "=== llm-task config (intake agent) ==="
cfg="$(cd "$(dirname "$0")/../config" && pwd)/openclaw-llm-task.json"
if [ -f "$cfg" ]; then
  if command -v jq >/dev/null 2>&1; then
    jq -e '.plugins.entries["llm-task"].enabled == true' "$cfg" >/dev/null || { echo "ERROR: llm-task plugin not enabled in $cfg" >&2; exit 1; }
    jq -e '.agents.list[] | select(.id == "intake") | .tools.allow | index("llm-task") != null' "$cfg" >/dev/null || {
      echo "ERROR: intake agent must allowlist llm-task in $cfg" >&2
      exit 1
    }
    jq -e '.agents.list[] | select(.id == "intake") | .tools.allow | index("lobster") != null' "$cfg" >/dev/null || {
      echo "WARN: intake agent should allowlist lobster for workflow runs" >&2
      code=1
    }
    echo "OK: $cfg (llm-task enabled + allowlisted for intake; lobster checked)"
  else
    echo "SKIP: jq not installed — manually verify $cfg matches https://docs.openclaw.ai/tools/llm-task"
  fi
else
  echo "ERROR: missing $cfg" >&2
  exit 1
fi

exit "$code"
