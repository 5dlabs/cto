#!/usr/bin/env bash
set -euo pipefail

KUBECONFIG_PATH="${KUBECONFIG_PATH:-/Users/jonathon/Library/Application Support/Lens/kubeconfigs/10f4c747-94d8-4d60-b40c-27caccd1b233-pasted-kubeconfig.yaml}"
NAMESPACE="${OPENCLAW_NAMESPACE:-openclaw}"
SERVICE_NAME="${OPENCLAW_SERVICE_NAME:-openclaw-optimist}"
LOCAL_PORT="${OPENCLAW_LOCAL_PORT:-33189}"
REMOTE_PORT="${OPENCLAW_REMOTE_PORT:-18789}"

export KUBECONFIG="${KUBECONFIG:-$KUBECONFIG_PATH}"

echo "Forwarding ${NAMESPACE}/${SERVICE_NAME} ${LOCAL_PORT} -> ${REMOTE_PORT}"
exec kubectl port-forward -n "$NAMESPACE" "svc/${SERVICE_NAME}" "${LOCAL_PORT}:${REMOTE_PORT}"
