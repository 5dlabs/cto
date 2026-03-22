#!/usr/bin/env bash
# Bootstrap OpenBao policy/role for External Secrets Operator.
#
# Purpose:
# - Ensure the kubernetes auth role used by ESO exists and is bound correctly.
# - Ensure the read policy includes all OpenBao paths ESO needs for bridge/runtime
#   secrets (including bots namespace bridge credentials).
#
# Usage:
#   ./infra/scripts/openbao/bootstrap-eso-policy.sh
#   ./infra/scripts/openbao/bootstrap-eso-policy.sh --namespace vault --pod openclaw-openbao-0
#
# Notes:
# - Uses the in-cluster OpenBao root token from secret/openbao-unseal-key.
# - Safe to run repeatedly (idempotent updates).

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

OPENBAO_NAMESPACE="${OPENBAO_NAMESPACE:-vault}"
OPENBAO_POD="${OPENBAO_POD:-openclaw-openbao-0}"
ROOT_SECRET_NAME="${ROOT_SECRET_NAME:-openbao-unseal-key}"
ROOT_SECRET_KEY="${ROOT_SECRET_KEY:-root-token}"

AUTH_MOUNT="${AUTH_MOUNT:-kubernetes}"
ESO_ROLE_NAME="${ESO_ROLE_NAME:-external-secrets}"
ESO_SERVICE_ACCOUNT="${ESO_SERVICE_ACCOUNT:-openclaw-external-secrets}"
ESO_SERVICE_ACCOUNT_NAMESPACE="${ESO_SERVICE_ACCOUNT_NAMESPACE:-external-secrets}"
POLICY_NAME="${POLICY_NAME:-openclaw-read}"

log() { echo -e "${BLUE}[INFO]${NC} $*" >&2; }
ok() { echo -e "${GREEN}[OK]${NC} $*" >&2; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
fail() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

usage() {
  cat <<'EOF'
Bootstrap OpenBao policy/role for External Secrets.

Options:
  --namespace <ns>              OpenBao namespace (default: vault)
  --pod <name>                  OpenBao pod name (default: openclaw-openbao-0)
  --root-secret-name <name>     Secret containing root token (default: openbao-unseal-key)
  --root-secret-key <key>       Key in secret containing root token (default: root-token)
  -h, --help                    Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --namespace) OPENBAO_NAMESPACE="$2"; shift 2 ;;
    --pod) OPENBAO_POD="$2"; shift 2 ;;
    --root-secret-name) ROOT_SECRET_NAME="$2"; shift 2 ;;
    --root-secret-key) ROOT_SECRET_KEY="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done

command -v kubectl >/dev/null 2>&1 || fail "kubectl is required"

log "Checking OpenBao pod availability"
kubectl -n "$OPENBAO_NAMESPACE" get pod "$OPENBAO_POD" >/dev/null

log "Reading OpenBao root token from ${OPENBAO_NAMESPACE}/${ROOT_SECRET_NAME}"
ROOT_TOKEN="$(
  kubectl -n "$OPENBAO_NAMESPACE" get secret "$ROOT_SECRET_NAME" \
    -o "jsonpath={.data.${ROOT_SECRET_KEY}}" | base64 -d
)"
[[ -n "$ROOT_TOKEN" ]] || fail "Root token is empty"

log "Writing policy ${POLICY_NAME}"
kubectl -n "$OPENBAO_NAMESPACE" exec -i "$OPENBAO_POD" -- \
  env BAO_TOKEN="$ROOT_TOKEN" bao policy write "$POLICY_NAME" - <<'EOF'
path "secret/data/openclaw/*" {
  capabilities = ["read", "list"]
}
path "secret/metadata/openclaw/*" {
  capabilities = ["read", "list"]
}

path "secret/data/openclaw-linear" {
  capabilities = ["read", "list"]
}
path "secret/metadata/openclaw-linear" {
  capabilities = ["read", "list"]
}

path "secret/data/openclaw-discord" {
  capabilities = ["read", "list"]
}
path "secret/metadata/openclaw-discord" {
  capabilities = ["read", "list"]
}

path "secret/data/ghcr-secret" {
  capabilities = ["read", "list"]
}
path "secret/metadata/ghcr-secret" {
  capabilities = ["read", "list"]
}
EOF

log "Upserting kubernetes auth role ${ESO_ROLE_NAME}"
kubectl -n "$OPENBAO_NAMESPACE" exec "$OPENBAO_POD" -- \
  env BAO_TOKEN="$ROOT_TOKEN" bao write "auth/${AUTH_MOUNT}/role/${ESO_ROLE_NAME}" \
  bound_service_account_names="$ESO_SERVICE_ACCOUNT" \
  bound_service_account_namespaces="$ESO_SERVICE_ACCOUNT_NAMESPACE" \
  policies="$POLICY_NAME" \
  ttl="1h" >/dev/null

log "Validating auth role and policy"
kubectl -n "$OPENBAO_NAMESPACE" exec "$OPENBAO_POD" -- \
  env BAO_TOKEN="$ROOT_TOKEN" bao read "auth/${AUTH_MOUNT}/role/${ESO_ROLE_NAME}" >/dev/null
kubectl -n "$OPENBAO_NAMESPACE" exec "$OPENBAO_POD" -- \
  env BAO_TOKEN="$ROOT_TOKEN" bao policy read "$POLICY_NAME" >/dev/null

ok "OpenBao policy/role bootstrap complete"
echo "Applied policy: ${POLICY_NAME}"
echo "Applied role: auth/${AUTH_MOUNT}/role/${ESO_ROLE_NAME}"
