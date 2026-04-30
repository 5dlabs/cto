#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ADAPTER="$ROOT/intake/scripts/acpx-llm-task.py"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }

FAKEBIN="$TMPDIR/bin"
mkdir -p "$FAKEBIN"
cat > "$FAKEBIN/acpx" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$ACPX_ARGV_FILE"
prompt=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "-f" ]; then prompt="$arg"; fi
  prev="$arg"
done
[ -n "$prompt" ] && cp "$prompt" "$ACPX_PROMPT_COPY"
if [ "${ACPX_FAKE_INVALID_JSON:-}" = "1" ]; then
  printf 'not json\n'
else
  printf '{"ok":true,"actual_provider":"github-copilot","actual_model":"gpt-5.5"}\n'
fi
SH
chmod +x "$FAKEBIN/acpx"
for c in copilot gemini claude codex opencode droid cursor; do
  cat > "$FAKEBIN/$c" <<'SH'
#!/usr/bin/env bash
printf 'fake agent\n'
SH
  chmod +x "$FAKEBIN/$c"
done

SCHEMA="$TMPDIR/schema.json"
printf '{"type":"object","properties":{"ok":{"type":"boolean"}}}\n' > "$SCHEMA"
PAYLOAD="$TMPDIR/payload.json"
python3 - "$PAYLOAD" "$SCHEMA" <<'PY'
import json, sys
path, schema = sys.argv[1:3]
json.dump({
  "provider":"github-copilot",
  "model":"gpt-5.5",
  "prompt":"Return ok true",
  "input":{"fleet":"axis"},
  "schema": schema,
}, open(path,'w'))
PY

ARGV="$TMPDIR/argv.txt"
PROMPT_COPY="$TMPDIR/prompt.md"
OUT=$(env -i PATH="$FAKEBIN:$PATH" ACPX_ARGV_FILE="$ARGV" ACPX_PROMPT_COPY="$PROMPT_COPY" WORKSPACE="$ROOT" "$ADAPTER" --tool llm-task --action json --args-file "$PAYLOAD")
printf '%s' "$OUT" | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["ok"] is True; assert data["actual_provider"] == "github-copilot"'
grep -q -- '--format' "$ARGV" || fail "ACPX argv includes --format"
grep -q -- 'text' "$ARGV" || fail "ACPX argv includes text format"
grep -q -- '--model' "$ARGV" || fail "ACPX argv includes --model"
grep -q -- 'gpt-5.5' "$ARGV" || fail "ACPX argv includes requested model"
grep -q -- 'copilot' "$ARGV" || fail "ACPX argv uses copilot agent"
grep -q -- 'exec' "$ARGV" || fail "ACPX argv uses exec"
grep -q -- '-f' "$ARGV" || fail "ACPX argv passes prompt file"
grep -q 'Return ok true' "$PROMPT_COPY" || fail "prompt includes task prompt"
grep -q '"fleet": "axis"' "$PROMPT_COPY" || fail "prompt includes input JSON"
grep -q 'Schema JSON' "$PROMPT_COPY" || fail "prompt includes schema section"
pass "acpx llm-task maps github-copilot payload to ACPX copilot exec and extracts JSON"

CAPS=$(env -i PATH="$FAKEBIN:$PATH" "$ADAPTER" --tool provider-capabilities --action json)
printf '%s' "$CAPS" | python3 -c 'import json,sys; data=json.load(sys.stdin); p=data["providers"]; assert p["github-copilot"]["available"] is True; assert p["github-copilot"]["invoke"] == "acpx"; assert p["gemini"]["available"] is True'
if printf '%s' "$CAPS" | grep -Eq 'TOKEN|KEY=|SECRET'; then
  printf '%s\n' "$CAPS" >&2
  fail "capabilities do not leak secrets"
fi
pass "acpx provider capabilities are non-secret and command-based"

set +e
BAD=$(env -i PATH="$FAKEBIN:$PATH" ACPX_ARGV_FILE="$ARGV" ACPX_PROMPT_COPY="$PROMPT_COPY" ACPX_FAKE_INVALID_JSON=1 "$ADAPTER" --tool llm-task --action json --args-file "$PAYLOAD" 2>&1)
STATUS=$?
set -e
[ "$STATUS" -ne 0 ] || { printf '%s\n' "$BAD" >&2; fail "invalid ACPX JSON exits nonzero"; }
printf '%s' "$BAD" | grep -qi 'valid JSON' || { printf '%s\n' "$BAD" >&2; fail "invalid JSON error is clear"; }
pass "invalid ACPX JSON exits nonzero with clear error"
