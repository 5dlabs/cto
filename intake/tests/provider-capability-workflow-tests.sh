#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PIPELINE="$ROOT/intake/workflows/pipeline.lobster.yaml"
ADAPTER="$ROOT/intake/scripts/real-llm-invoke.py"

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }

# Pipeline should expose provider capabilities in load-config output so runs can
# distinguish requested role routing from actually callable backends.
grep -q 'provider_capabilities' "$PIPELINE" || fail "load-config emits provider capabilities"
grep -q 'provider-capabilities' "$PIPELINE" || fail "load-config calls provider-capabilities tool"
pass "load-config emits provider capabilities"

# Adapter should support a safe capabilities probe.
CAPS=$(env -i PATH="$PATH" "$ADAPTER" --tool provider-capabilities --action json)
printf '%s' "$CAPS" | python3 -c 'import json,sys; data=json.load(sys.stdin); assert "github-copilot" in data["providers"]; assert "openai" in data["providers"]; assert "anthropic" in data["providers"]; assert "gemini" in data["providers"]'
pass "adapter exposes all provider capability records"
