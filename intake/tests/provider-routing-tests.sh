#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ADAPTER="$ROOT/intake/scripts/real-llm-invoke.py"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }

payload() {
  local req_provider="$1" req_model="$2" path
  path="$TMPDIR/${req_provider}-${req_model}.json"
  python3 - "$req_provider" "$req_model" "$path" <<'PY'
import json, sys
provider, model, path = sys.argv[1:4]
json.dump({
    "provider": provider,
    "model": model,
    "prompt": "Return {\"ok\": true}",
    "input": {"ping": "pong"},
}, open(path, "w"))
PY
  printf '%s' "$path"
}

# Explicit github-copilot requests must not silently fall back to REAL_LLM_PROVIDER.
grep -q '"github-copilot/gpt-5.5"' "$ROOT/intake/config/openclaw-llm-task.json" || fail "OpenClaw llm-task allowlist includes github-copilot/gpt-5.5"
pass "OpenClaw llm-task allowlist includes github-copilot/gpt-5.5"

COPILOT_PAYLOAD="$(payload github-copilot gpt-5.5)"
set +e
OUT=$(env -i PATH="$PATH" REAL_LLM_PROVIDER=gemini REAL_LLM_MODEL=gemini-3.1-pro-preview "$ADAPTER" --tool llm-task --action json --args-file "$COPILOT_PAYLOAD" 2>&1)
STATUS=$?
set -e
if [ "$STATUS" -eq 0 ]; then
  printf '%s\n' "$OUT" >&2
  fail "explicit github-copilot request fails closed when no Copilot harness is configured"
fi
printf '%s' "$OUT" | grep -q 'provider_unavailable' || { printf '%s\n' "$OUT" >&2; fail "copilot failure reports provider_unavailable"; }
printf '%s' "$OUT" | grep -q 'github-copilot' || { printf '%s\n' "$OUT" >&2; fail "copilot failure names requested provider"; }
pass "explicit github-copilot request fails closed when no Copilot harness is configured"

# Explicit github-copilot can be served by a configured provider command and reports text to stdout.
HARNESS="$TMPDIR/fake-copilot-harness.sh"
cat > "$HARNESS" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import json
print(json.dumps({"ok": True, "actual_provider": "github-copilot", "actual_model": "gpt-5.5"}))
PY
SH
chmod +x "$HARNESS"
OUT=$(env -i PATH="$PATH" COPILOT_LLM_INVOKE_CMD="$HARNESS" "$ADAPTER" --tool llm-task --action json --args-file "$COPILOT_PAYLOAD")
printf '%s' "$OUT" | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["actual_provider"] == "github-copilot" and data["actual_model"] == "gpt-5.5"'
pass "explicit github-copilot request can use configured Copilot harness"

# Capability check should be safe and non-secret.
CAPS=$(env -i PATH="$PATH" COPILOT_LLM_INVOKE_CMD="$HARNESS" OPENAI_API_KEY=dummy ANTHROPIC_API_KEY=dummy GEMINI_API_KEY=dummy "$ADAPTER" --tool provider-capabilities --action json)
printf '%s' "$CAPS" | python3 -c 'import json,sys; data=json.load(sys.stdin); providers=data["providers"]; assert providers["github-copilot"]["available"] is True; assert providers["openai"]["available"] is True; assert providers["anthropic"]["available"] is True; assert providers["gemini"]["available"] is True'
if printf '%s' "$CAPS" | grep -Eq 'dummy|TOKEN|KEY='; then
  printf '%s\n' "$CAPS" >&2
  fail "capability output does not expose secret values"
fi
pass "provider capabilities include supported providers without secrets"
