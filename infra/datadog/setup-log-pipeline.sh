#!/usr/bin/env bash
# Creates or updates DD Log Pipelines for CTO platform.
# Handles both CTO CodeRun and OpenClaw Agent pipelines.
# Requires DD_API_KEY and DD_APP_KEY environment variables.
#
# Usage:
#   DD_API_KEY=xxx DD_APP_KEY=yyy ./setup-log-pipeline.sh
#   # or with 1Password:
#   op run --env-file ./dd.env -- ./infra/datadog/setup-log-pipeline.sh
set -euo pipefail

DD_SITE="${DD_SITE:-us5.datadoghq.com}"
SCRIPT_DIR="$(dirname "$0")"

if [ -z "${DD_API_KEY:-}" ] || [ -z "${DD_APP_KEY:-}" ]; then
  echo "❌ DD_API_KEY and DD_APP_KEY must be set"
  echo ""
  echo "Create an Application Key at:"
  echo "  https://${DD_SITE}/organization-settings/application-keys"
  exit 1
fi

# Upsert a single pipeline by name
upsert_pipeline() {
  local pipeline_name="$1"
  local pipeline_file="$2"

  echo "🔍 Checking for existing '${pipeline_name}' pipeline..."

  EXISTING=$(curl -s -w "\n%{http_code}" "https://api.${DD_SITE}/api/v1/logs/config/pipelines" \
    -H "DD-API-KEY: ${DD_API_KEY}" \
    -H "DD-APPLICATION-KEY: ${DD_APP_KEY}")
  HTTP_CODE=$(echo "$EXISTING" | tail -1)
  BODY=$(echo "$EXISTING" | sed '$d')

  if [ "$HTTP_CODE" != "200" ]; then
    echo "❌ Failed to list pipelines (HTTP ${HTTP_CODE})"
    echo "$BODY" | python3 -m json.tool 2>/dev/null || echo "$BODY"
    return 1
  fi

  PIPELINE_ID=$(echo "$BODY" | python3 -c "
import json, sys
pipelines = json.load(sys.stdin)
for p in pipelines:
    if p.get('name') == '${pipeline_name}':
        print(p['id'])
        break
" 2>/dev/null || true)

  if [ -n "$PIPELINE_ID" ]; then
    echo "📝 Updating existing pipeline: $PIPELINE_ID"
    RESP=$(curl -s -w "\n%{http_code}" -X PUT "https://api.${DD_SITE}/api/v1/logs/config/pipelines/${PIPELINE_ID}" \
      -H "DD-API-KEY: ${DD_API_KEY}" \
      -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
      -H "Content-Type: application/json" \
      -d @"$pipeline_file")
  else
    echo "🆕 Creating new pipeline..."
    RESP=$(curl -s -w "\n%{http_code}" -X POST "https://api.${DD_SITE}/api/v1/logs/config/pipelines" \
      -H "DD-API-KEY: ${DD_API_KEY}" \
      -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
      -H "Content-Type: application/json" \
      -d @"$pipeline_file")
    PIPELINE_ID=$(echo "$RESP" | sed '$d' | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || true)
  fi

  HTTP_CODE=$(echo "$RESP" | tail -1)
  BODY=$(echo "$RESP" | sed '$d')

  if [ "$HTTP_CODE" -lt 200 ] || [ "$HTTP_CODE" -ge 300 ]; then
    echo "❌ Pipeline upsert failed (HTTP ${HTTP_CODE})"
    echo "$BODY" | python3 -m json.tool 2>/dev/null || echo "$BODY"
    return 1
  fi

  # Check for application-level errors
  ERROR=$(echo "$BODY" | python3 -c "
import json, sys
d = json.load(sys.stdin)
errs = d.get('errors', [])
if errs:
    print('; '.join(str(e) for e in errs))
" 2>/dev/null || true)

  if [ -n "$ERROR" ]; then
    echo "❌ Error: $ERROR"
    echo "$BODY" | python3 -m json.tool 2>/dev/null || echo "$BODY"
    return 1
  fi

  echo "✅ Pipeline ready: $PIPELINE_ID"
  echo "   View at: https://${DD_SITE}/logs/pipelines/pipeline/${PIPELINE_ID}"
}

# --- Pipeline 1: CTO CodeRun ---
echo "═══ CTO CodeRun Pipeline ═══"
upsert_pipeline "CTO CodeRun Logs" "${SCRIPT_DIR}/cto-coderun-pipeline.json"
echo ""

# --- Pipeline 2: OpenClaw Agent ---
echo "═══ OpenClaw Agent Pipeline ═══"
upsert_pipeline "OpenClaw Agent Logs" "${SCRIPT_DIR}/openclaw-agent-pipeline.json"
echo ""

# --- Facets ---
echo "📊 Creating facets..."
FACETS=(
  "acp.type" "acp.session_id" "acp.category" "acp.cli_version"
  "acp.entrypoint" "git.branch"
  "openclaw.subsystem" "openclaw.component" "openclaw.error_type"
  "openclaw.session_key" "openclaw.provider"
)
for FACET_PATH in "${FACETS[@]}"; do
  FACET_NAME=$(echo "$FACET_PATH" | sed 's/.*\.//')
  RESP=$(curl -s -w "\n%{http_code}" -X POST "https://api.${DD_SITE}/api/v1/logs/config/indexes" \
    -H "DD-API-KEY: ${DD_API_KEY}" \
    -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
    -H "Content-Type: application/json" \
    -d "{
      \"facet\": {
        \"path\": \"@${FACET_PATH}\",
        \"source\": \"log\",
        \"name\": \"${FACET_NAME}\",
        \"type\": \"string\"
      }
    }" 2>/dev/null || true)
  echo "  ✓ @${FACET_PATH}"
done

echo ""
echo "🎉 Done! Check your logs at:"
echo "   CodeRun:  https://${DD_SITE}/logs?query=source%3Acto-coderun"
echo "   Agents:   https://${DD_SITE}/logs?query=source%3Aopenclaw-agent"
echo "   Coder:    https://${DD_SITE}/logs?query=source%3Aopenclaw-agent%20service%3Acto-coder"
