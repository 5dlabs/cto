#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCRIPT="$ROOT/intake/scripts/llm-invoke.sh"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }

make_cmd() {
  local path="$1"
  local label="$2"
  cat > "$path" <<SH
#!/usr/bin/env bash
printf '%s\n' '$label' > "\$LLM_BACKEND_MARKER"
printf '{"backend":"$label"}\n'
SH
  chmod +x "$path"
}

FAKEBIN="$TMPDIR/bin"
mkdir -p "$FAKEBIN"
cat > "$FAKEBIN/acpx" <<'SH'
#!/usr/bin/env bash
printf 'fake acpx\n'
SH
chmod +x "$FAKEBIN/acpx"

# Explicit command always wins.
EXPLICIT="$TMPDIR/explicit.sh"
make_cmd "$EXPLICIT" explicit
MARKER="$TMPDIR/marker-explicit"
OUT=$(env -i PATH="$PATH" WORKSPACE="$ROOT" LLM_BACKEND_MARKER="$MARKER" CTO_LLM_INVOKE_CMD="$EXPLICIT" "$SCRIPT" --tool provider-capabilities --action json)
[ "$(cat "$MARKER")" = "explicit" ] || fail "explicit CTO_LLM_INVOKE_CMD is used"
printf '%s' "$OUT" | grep -q '"backend":"explicit"' || fail "explicit output returned"
pass "explicit CTO_LLM_INVOKE_CMD remains highest priority"

# ACP backend can be forced and routes to repo adapter. WORKSPACE may point at
# the broader persistent workspace, so llm-invoke must locate repo-local scripts
# from its own path, not from WORKSPACE.
ACP_STUB="$ROOT/intake/scripts/acpx-llm-task.py"
BACKUP=""
if [ -e "$ACP_STUB" ]; then
  BACKUP="$TMPDIR/acpx-llm-task.py.backup"
  cp "$ACP_STUB" "$BACKUP"
fi
cat > "$ACP_STUB" <<'SH'
#!/usr/bin/env bash
printf 'acp\n' > "$LLM_BACKEND_MARKER"
printf '{"backend":"acp"}\n'
SH
chmod +x "$ACP_STUB"
restore_stub() {
  if [ -n "$BACKUP" ]; then cp "$BACKUP" "$ACP_STUB"; else rm -f "$ACP_STUB"; fi
}
trap 'restore_stub; rm -rf "$TMPDIR"' EXIT

MARKER="$TMPDIR/marker-acp-forced"
OUT=$(env -i PATH="$FAKEBIN:$PATH" WORKSPACE="$(dirname "$ROOT")" LLM_BACKEND_MARKER="$MARKER" CTO_LLM_INVOKE_BACKEND=acp "$SCRIPT" --tool provider-capabilities --action json)
[ "$(cat "$MARKER")" = "acp" ] || fail "forced acp backend is used when WORKSPACE is the parent workspace"
printf '%s' "$OUT" | grep -q '"backend":"acp"' || fail "forced acp output returned"
pass "CTO_LLM_INVOKE_BACKEND=acp uses repo-local adapter independent of WORKSPACE"

# Auto prefers ACP when acpx exists.
MARKER="$TMPDIR/marker-auto"
OUT=$(env -i PATH="$FAKEBIN:$PATH" WORKSPACE="$ROOT" LLM_BACKEND_MARKER="$MARKER" CTO_LLM_INVOKE_BACKEND=auto "$SCRIPT" --tool provider-capabilities --action json)
[ "$(cat "$MARKER")" = "acp" ] || fail "auto backend prefers ACP when acpx is available"
pass "auto backend prefers ACPX adapter when available"

# Direct backend uses real direct adapter only when explicitly requested.
REAL="$ROOT/intake/scripts/real-llm-invoke.py"
REAL_BACKUP=""
if [ -e "$REAL" ]; then
  REAL_BACKUP="$TMPDIR/real-llm-invoke.py.backup"
  cp "$REAL" "$REAL_BACKUP"
fi
cat > "$REAL" <<'SH'
#!/usr/bin/env bash
printf 'direct\n' > "$LLM_BACKEND_MARKER"
printf '{"backend":"direct"}\n'
SH
chmod +x "$REAL"
restore_all() {
  if [ -n "$BACKUP" ]; then cp "$BACKUP" "$ACP_STUB"; else rm -f "$ACP_STUB"; fi
  if [ -n "$REAL_BACKUP" ]; then cp "$REAL_BACKUP" "$REAL"; else rm -f "$REAL"; fi
  rm -rf "$TMPDIR"
}
trap 'restore_all' EXIT
MARKER="$TMPDIR/marker-direct"
OUT=$(env -i PATH="$PATH" WORKSPACE="$ROOT" LLM_BACKEND_MARKER="$MARKER" CTO_LLM_INVOKE_BACKEND=direct "$SCRIPT" --tool provider-capabilities --action json)
[ "$(cat "$MARKER")" = "direct" ] || fail "direct backend is used only when requested"
pass "CTO_LLM_INVOKE_BACKEND=direct uses real llm invoke adapter"

# No backend yields clear error.
rm -f "$ACP_STUB"
set +e
ERR=$(env -i PATH="/usr/bin:/bin" WORKSPACE="$ROOT" CTO_LLM_INVOKE_BACKEND=acp "$SCRIPT" --tool provider-capabilities --action json 2>&1 >/dev/null)
STATUS=$?
set -e
[ "$STATUS" -ne 0 ] || fail "missing forced acp backend fails"
printf '%s' "$ERR" | grep -q 'CTO_LLM_INVOKE_BACKEND' || { printf '%s\n' "$ERR" >&2; fail "error mentions backend env"; }
printf '%s' "$ERR" | grep -q 'CTO_LLM_INVOKE_CMD' || { printf '%s\n' "$ERR" >&2; fail "error mentions explicit command"; }
pass "missing backend fails with setup guidance"
