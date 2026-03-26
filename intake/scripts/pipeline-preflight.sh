#!/usr/bin/env bash
# Verify infrastructure and env before pipeline.lobster.yaml side effects (Linear, cluster, bridges).
#
# Override URLs for Twingate / port-forwards:
#   DISCORD_BRIDGE_URL   (default http://discord-bridge.bots.svc:3200)
#   LINEAR_BRIDGE_URL    (default http://linear-bridge.bots.svc:3100)
#
# Escape hatches:
#   INTAKE_PREFLIGHT_SKIP=true   — no checks (local hack only)
#   INTAKE_PREFLIGHT_KUBECTL_SKIP=true — do not require kubectl (not recommended for full runs)
#   INTAKE_PREFLIGHT_BRIDGES_SKIP=true — skip Discord + Linear bridge /health curls only (still checks LINEAR_API_KEY + kubectl unless skipped)
#   INTAKE_OP_AUTO_DISABLE=1 — do not re-exec under `op run` (see intake/local.env.op.example)

set -euo pipefail

if [[ "${INTAKE_PREFLIGHT_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_SKIP:-}" == "true" ]]; then
  echo "intake/scripts/pipeline-preflight.sh: INTAKE_PREFLIGHT_SKIP set — skipping checks" >&2
  exit 0
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
"$ROOT/intake/scripts/ensure-local-env-op.sh" || true
# shellcheck source=intake/scripts/intake-op-auto.sh
source "$ROOT/intake/scripts/intake-op-auto.sh"
intake_op_auto_wrap "${BASH_SOURCE[0]}" "$@"

fail() {
  echo "preflight FAILED: $*" >&2
  exit 1
}

CURL_OPTS=(--silent --show-error --fail --max-time "${INTAKE_PREFLIGHT_CURL_TIMEOUT:-10}")

DISCORD_URL="${DISCORD_BRIDGE_URL:-http://discord-bridge.bots.svc:3200}"
LINEAR_URL="${LINEAR_BRIDGE_URL:-http://linear-bridge.bots.svc:3100}"
# Normalize: no trailing slash
DISCORD_URL="${DISCORD_URL%/}"
LINEAR_URL="${LINEAR_URL%/}"

[[ -n "${LINEAR_API_KEY:-}" ]] || fail "LINEAR_API_KEY is unset or empty — copy intake/local.env.op.example → intake/local.env.op with op:// reference, or export manually (see docs/intake-local-prereqs.md)"

if [[ "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "true" ]]; then
  echo "preflight WARNING: INTAKE_PREFLIGHT_BRIDGES_SKIP — skipping bridge /health (notify/register steps still need reachable DISCORD_BRIDGE_URL / LINEAR_BRIDGE_URL at runtime)" >&2
else
  curl "${CURL_OPTS[@]}" "${DISCORD_URL}/health" >/dev/null \
    || fail "DISCORD_BRIDGE_URL not reachable at ${DISCORD_URL}/health — check Twingate, DNS, service up, or port-forward"

  curl "${CURL_OPTS[@]}" "${LINEAR_URL}/health" >/dev/null \
    || fail "LINEAR_BRIDGE_URL not reachable at ${LINEAR_URL}/health — check Twingate / port-forward"
fi

if [[ "${INTAKE_PREFLIGHT_KUBECTL_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_KUBECTL_SKIP:-}" == "true" ]]; then
  echo "preflight: kubectl check skipped (INTAKE_PREFLIGHT_KUBECTL_SKIP)" >&2
else
  command -v kubectl >/dev/null 2>&1 || fail "kubectl not on PATH"
  kubectl cluster-info --request-timeout=15s >/dev/null \
    || fail "kubectl cluster-info failed — set kube context for CTO cluster (Twingate to API if required)"
fi

if [[ "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "true" ]]; then
  echo "preflight OK: LINEAR_API_KEY set, bridge /health skipped, kubectl per flags" >&2
else
  echo "preflight OK: LINEAR_API_KEY set, ${DISCORD_URL}/health, ${LINEAR_URL}/health, kubectl" >&2
fi
