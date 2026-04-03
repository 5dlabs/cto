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
#   INTAKE_PREFLIGHT_TOKENS_SKIP=true — skip token/credential validation (use when running with minimal providers)
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

[[ -n "${LINEAR_API_KEY:-}" ]] || fail "LINEAR_API_KEY is unset or empty — export a PM-minted runtime token (preferred) or use the temporary intake/local.env.op fallback (see docs/intake-local-prereqs.md)"

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

## ── Token / credential validation ──────────────────────────────────────────
if [[ "${INTAKE_PREFLIGHT_TOKENS_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_TOKENS_SKIP:-}" == "true" ]]; then
  echo "preflight: token validation skipped (INTAKE_PREFLIGHT_TOKENS_SKIP)" >&2
else
  TOKEN_REPORT=""
  TOKEN_WARNINGS=""
  TOKEN_ERRORS=""

  # GitHub token — required for PR creation and github-copilot LLM provider
  if [[ -n "${GH_TOKEN:-}${GITHUB_TOKEN:-}" ]]; then
    TOKEN_REPORT="${TOKEN_REPORT}  ✓ GitHub token (GH_TOKEN/GITHUB_TOKEN)\n"
  else
    TOKEN_ERRORS="${TOKEN_ERRORS}  ✗ GH_TOKEN or GITHUB_TOKEN — required for PR creation and github-copilot LLM provider\n"
  fi

  # LLM API keys — at least one must be set for the cascade to work
  LLM_KEYS_SET=0
  LLM_KEYS_DETAIL=""
  for key_name in OPENROUTER_API_KEY GEMINI_API_KEY MINIMAX_API_KEY ANTHROPIC_API_KEY OPENAI_API_KEY; do
    val=""
    eval "val=\${${key_name}:-}"
    if [[ -n "$val" ]]; then
      LLM_KEYS_SET=$((LLM_KEYS_SET + 1))
      LLM_KEYS_DETAIL="${LLM_KEYS_DETAIL}  ✓ ${key_name}\n"
    else
      TOKEN_WARNINGS="${TOKEN_WARNINGS}  ⚠ ${key_name} unset — this fallback provider will be unavailable\n"
    fi
  done

  if [[ "$LLM_KEYS_SET" -eq 0 && -z "${GH_TOKEN:-}${GITHUB_TOKEN:-}" ]]; then
    TOKEN_ERRORS="${TOKEN_ERRORS}  ✗ No LLM provider credentials — need GH_TOKEN/GITHUB_TOKEN (for github-copilot) or at least one of: OPENROUTER_API_KEY, GEMINI_API_KEY, MINIMAX_API_KEY, ANTHROPIC_API_KEY, OPENAI_API_KEY\n"
  fi
  TOKEN_REPORT="${TOKEN_REPORT}${LLM_KEYS_DETAIL}"

  # Discord bridge token — optional but needed for notifications
  if [[ -n "${DISCORD_BRIDGE_TOKEN:-}" ]]; then
    TOKEN_REPORT="${TOKEN_REPORT}  ✓ DISCORD_BRIDGE_TOKEN\n"
  else
    TOKEN_WARNINGS="${TOKEN_WARNINGS}  ⚠ DISCORD_BRIDGE_TOKEN unset — Discord notifications will fail\n"
  fi

  # Stitch API key — optional, needed for design mode
  if [[ -n "${STITCH_API_KEY:-}" ]]; then
    TOKEN_REPORT="${TOKEN_REPORT}  ✓ STITCH_API_KEY\n"
  else
    TOKEN_WARNINGS="${TOKEN_WARNINGS}  ⚠ STITCH_API_KEY unset — Stitch design features will be unavailable\n"
  fi

  # OpenClaw gateway — check it's responding
  GATEWAY_URL="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18789/tools/invoke}"
  GATEWAY_HEALTH="${GATEWAY_URL%/tools/invoke}/health"
  if curl --silent --show-error --fail --max-time 5 "$GATEWAY_HEALTH" >/dev/null 2>&1; then
    TOKEN_REPORT="${TOKEN_REPORT}  ✓ OpenClaw gateway (${GATEWAY_HEALTH})\n"
  else
    # Try the base URL (some versions respond at root)
    GATEWAY_BASE="${GATEWAY_URL%/tools/invoke}"
    if curl --silent --show-error --max-time 5 "$GATEWAY_BASE" >/dev/null 2>&1; then
      TOKEN_REPORT="${TOKEN_REPORT}  ✓ OpenClaw gateway (${GATEWAY_BASE})\n"
    else
      TOKEN_WARNINGS="${TOKEN_WARNINGS}  ⚠ OpenClaw gateway not reachable at ${GATEWAY_HEALTH} — direct LLM calls will still work via fallback providers\n"
    fi
  fi

  # Print report
  echo "" >&2
  echo "preflight: Token / credential validation:" >&2
  [[ -n "$TOKEN_REPORT" ]] && printf '%b' "$TOKEN_REPORT" >&2
  if [[ -n "$TOKEN_WARNINGS" ]]; then
    echo "" >&2
    echo "  Warnings:" >&2
    printf '%b' "$TOKEN_WARNINGS" >&2
  fi
  if [[ -n "$TOKEN_ERRORS" ]]; then
    echo "" >&2
    printf '%b' "$TOKEN_ERRORS" >&2
    fail "Missing required tokens — see above. Set INTAKE_PREFLIGHT_TOKENS_SKIP=true to bypass."
  fi
  echo "" >&2
fi

## ── Summary ───────────────────────────────────────────────────────────────
if [[ "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "1" || "${INTAKE_PREFLIGHT_BRIDGES_SKIP:-}" == "true" ]]; then
  echo "preflight OK: LINEAR_API_KEY set, bridge /health skipped, kubectl per flags, tokens validated" >&2
else
  echo "preflight OK: LINEAR_API_KEY set, ${DISCORD_URL}/health, ${LINEAR_URL}/health, kubectl, tokens validated" >&2
fi
