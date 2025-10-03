#!/bin/bash
set -euo pipefail

# Render the agent templates ConfigMaps via Helm and apply them to the cluster.
# This script handles multiple split ConfigMaps (shared, claude, codex, cursor, factory, opencode).
# Optional environment overrides:
#   RELEASE_NAME  - Helm release name (default: controller)
#   NAMESPACE     - Kubernetes namespace (default: agent-platform)
#   VALUES_FILE   - Helm values file to include (default: infra/charts/controller/values.yaml)
# Additional Helm args can be passed after a double dash, e.g.:
#   ./scripts/apply-agent-templates-configmap.sh -- -f custom-values.yaml

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CHART_DIR="$ROOT_DIR/infra/charts/controller"

RELEASE_NAME=${RELEASE_NAME:-controller}
NAMESPACE=${NAMESPACE:-agent-platform}
VALUES_FILE_DEFAULT="$CHART_DIR/values.yaml"
VALUES_FILE=${VALUES_FILE:-$VALUES_FILE_DEFAULT}

# Split ConfigMap template names
CONFIGMAP_TEMPLATES=(
  "agent-templates-shared.yaml"
  "agent-templates-claude.yaml"
  "agent-templates-codex.yaml"
  "agent-templates-cursor.yaml"
  "agent-templates-factory.yaml"
  "agent-templates-opencode.yaml"
)

# Split optional extra Helm args after "--"
HELM_ARGS=()
if [[ $# -gt 0 ]]; then
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --)
        shift
        HELM_ARGS=("$@")
        break
        ;;
      *)
        echo "Unknown argument: $1" >&2
        echo "Usage: $0 [-- <additional helm args>]" >&2
        exit 1
        ;;
    esac
  done
fi

if ! command -v helm >/dev/null 2>&1; then
  echo "❌ Helm must be installed to render the ConfigMaps" >&2
  exit 1
fi

echo "🚀 Rendering and applying agent templates ConfigMaps..."
echo ""

# Process each ConfigMap
for template in "${CONFIGMAP_TEMPLATES[@]}"; do
  TMP_FILE=$(mktemp)
  
  echo "📦 Processing: $template"
  
  if [[ ${#HELM_ARGS[@]} -gt 0 ]]; then
    helm template "$RELEASE_NAME" "$CHART_DIR" \
      --namespace "$NAMESPACE" \
      --values "$VALUES_FILE" \
      --show-only "templates/$template" \
      "${HELM_ARGS[@]}" > "$TMP_FILE"
  else
    helm template "$RELEASE_NAME" "$CHART_DIR" \
      --namespace "$NAMESPACE" \
      --values "$VALUES_FILE" \
      --show-only "templates/$template" > "$TMP_FILE"
  fi
  
  kubectl apply --server-side --force-conflicts -f "$TMP_FILE"
  echo "✅ Applied: $template"
  echo ""
  
  rm -f "$TMP_FILE"
done

echo "✅ All agent templates ConfigMaps applied successfully!"
