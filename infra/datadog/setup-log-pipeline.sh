#!/usr/bin/env bash
# Creates or updates the DD Log Pipeline for CTO CodeRun logs.
# Requires DD_API_KEY and DD_APP_KEY environment variables.
#
# Usage:
#   DD_API_KEY=xxx DD_APP_KEY=yyy ./setup-log-pipeline.sh
#   # or with 1Password:
#   op run --env-file ./dd.env -- ./infra/datadog/setup-log-pipeline.sh
set -euo pipefail

DD_SITE="${DD_SITE:-us5.datadoghq.com}"
PIPELINE_FILE="$(dirname "$0")/cto-coderun-pipeline.json"

if [ -z "${DD_API_KEY:-}" ] || [ -z "${DD_APP_KEY:-}" ]; then
  echo "❌ DD_API_KEY and DD_APP_KEY must be set"
  echo ""
  echo "Create an Application Key at:"
  echo "  https://${DD_SITE}/organization-settings/application-keys"
  exit 1
fi

echo "🔍 Checking for existing 'CTO CodeRun Logs' pipeline..."

# List existing pipelines
EXISTING=$(curl -s "https://api.${DD_SITE}/api/v1/logs/config/pipelines" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -H "DD-APPLICATION-KEY: ${DD_APP_KEY}")

PIPELINE_ID=$(echo "$EXISTING" | python3 -c "
import json, sys
pipelines = json.load(sys.stdin)
for p in pipelines:
    if p.get('name') == 'CTO CodeRun Logs':
        print(p['id'])
        break
" 2>/dev/null || true)

if [ -n "$PIPELINE_ID" ]; then
  echo "📝 Updating existing pipeline: $PIPELINE_ID"
  RESP=$(curl -s -X PUT "https://api.${DD_SITE}/api/v1/logs/config/pipelines/${PIPELINE_ID}" \
    -H "DD-API-KEY: ${DD_API_KEY}" \
    -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
    -H "Content-Type: application/json" \
    -d @"$PIPELINE_FILE")
else
  echo "🆕 Creating new pipeline..."
  RESP=$(curl -s -X POST "https://api.${DD_SITE}/api/v1/logs/config/pipelines" \
    -H "DD-API-KEY: ${DD_API_KEY}" \
    -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
    -H "Content-Type: application/json" \
    -d @"$PIPELINE_FILE")
  PIPELINE_ID=$(echo "$RESP" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || true)
fi

# Check for errors
ERROR=$(echo "$RESP" | python3 -c "
import json, sys
d = json.load(sys.stdin)
errs = d.get('errors', [])
if errs:
    print('; '.join(str(e) for e in errs))
" 2>/dev/null || true)

if [ -n "$ERROR" ]; then
  echo "❌ Error: $ERROR"
  echo "$RESP" | python3 -m json.tool 2>/dev/null || echo "$RESP"
  exit 1
fi

echo "✅ Pipeline ready: $PIPELINE_ID"
echo "   View at: https://${DD_SITE}/logs/pipelines/pipeline/${PIPELINE_ID}"

# Now create facets for the key attributes
echo ""
echo "📊 Creating facets for ACP attributes..."
for FACET_PATH in "acp.type" "acp.session_id" "acp.category" "acp.cli_version" \
                  "acp.entrypoint" "git.branch" "openclaw.subsystem"; do
  FACET_NAME=$(echo "$FACET_PATH" | sed 's/.*\.//')
  GROUP=$(echo "$FACET_PATH" | sed 's/\..*//')

  curl -s -X POST "https://api.${DD_SITE}/api/v1/logs/config/indexes" \
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
    }" > /dev/null 2>&1 || true
  echo "  ✓ @${FACET_PATH}"
done

echo ""
echo "🎉 Done! Check your logs at:"
echo "   https://${DD_SITE}/logs?query=source%3Acto-coderun"
