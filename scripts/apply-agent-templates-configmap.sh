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
  
  # Extract ConfigMap name using yq for robust YAML parsing
  # Falls back to grep if yq is not available
  if command -v yq >/dev/null 2>&1; then
    CM_NAME=$(yq eval '.metadata.name' "$TMP_FILE")
  else
    # Fallback: Use grep with more robust pattern
    # Match "name:" at any indentation level under metadata
    CM_NAME=$(grep -E '^\s*name:\s*\S+' "$TMP_FILE" | grep -v "kind:" | head -1 | sed -E 's/^\s*name:\s*//')
  fi
  
  # Validate that we extracted a non-empty name
  if [[ -z "$CM_NAME" ]] || [[ "$CM_NAME" == "null" ]]; then
    echo "❌ Failed to extract ConfigMap name from $template" >&2
    echo "   Check the YAML structure in the rendered template" >&2
    echo "   Template content:" >&2
    head -20 "$TMP_FILE" >&2
    rm -f "$TMP_FILE"
    exit 1
  fi
  
  echo "   ConfigMap name: $CM_NAME"
  
  # FORCE DELETE/RECREATE instead of patch to guarantee fresh content
  echo "🗑️  Force deleting: $CM_NAME"
  kubectl delete configmap "$CM_NAME" -n "$NAMESPACE" --ignore-not-found
  sleep 2
  
  # Create with retry logic
  MAX_RETRIES=3
  RETRY=0
  SUCCESS=false
  
  while [ $RETRY -lt $MAX_RETRIES ]; do
    if kubectl create -f "$TMP_FILE" 2>&1; then
      echo "✅ Applied: $template"
      SUCCESS=true
      break
    else
      RETRY=$((RETRY + 1))
      if [ $RETRY -lt $MAX_RETRIES ]; then
        WAIT=$((2 ** RETRY))
        echo "⚠️  Attempt $RETRY failed, retrying in ${WAIT}s..."
        sleep $WAIT
      fi
    fi
  done
  
  if [ "$SUCCESS" = "false" ]; then
    echo "❌ Failed to create $template after $MAX_RETRIES attempts"
    rm -f "$TMP_FILE"
    exit 1
  fi
  
  rm -f "$TMP_FILE"
  echo ""
done

echo "✅ All agent templates ConfigMaps applied successfully!"

