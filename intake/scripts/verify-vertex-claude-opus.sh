#!/usr/bin/env bash
# Smoke-test Claude Opus 4.6 on Vertex AI (partner / Anthropic publisher model).
#
# This does NOT use GEMINI_API_KEY (that key is for Google AI Studio / generativelanguage.googleapis.com).
# Vertex requires GCP auth: `gcloud auth login` and/or `gcloud auth application-default login`.
#
# Usage:
#   export GOOGLE_CLOUD_PROJECT=my-project   # optional if gcloud default project is set
#   export VERTEX_LOCATION=us-east5          # optional; script tries a few common regions
#   ./intake/scripts/verify-vertex-claude-opus.sh
#
# Docs: https://docs.anthropic.com/en/api/claude-on-vertex-ai
set -euo pipefail

PROJECT="${GOOGLE_CLOUD_PROJECT:-$(gcloud config get-value project 2>/dev/null || true)}"
if [ -z "$PROJECT" ] || [ "$PROJECT" = "(unset)" ]; then
  echo "Set GOOGLE_CLOUD_PROJECT or run: gcloud config set project YOUR_PROJECT_ID" >&2
  exit 1
fi

if ! TOKEN="$(gcloud auth print-access-token 2>/dev/null)"; then
  echo "gcloud has no credentials. Run: gcloud auth login && gcloud auth application-default login" >&2
  exit 1
fi

MODEL_ID="${VERTEX_CLAUDE_MODEL:-claude-opus-4-6}"
# Regions where Anthropic on Vertex is commonly available (adjust if your admin enabled others)
REGIONS_TO_TRY=("${VERTEX_LOCATION:-}" us-east5 us-central1 europe-west1)
BODY='{"anthropic_version":"vertex-2023-10-30","max_tokens":64,"messages":[{"role":"user","content":"Reply with exactly the word: ok"}]}'

for LOC in "${REGIONS_TO_TRY[@]}"; do
  [ -z "$LOC" ] && continue
  URL="https://${LOC}-aiplatform.googleapis.com/v1/projects/${PROJECT}/locations/${LOC}/publishers/anthropic/models/${MODEL_ID}:rawPredict"
  echo "Trying region=${LOC} model=${MODEL_ID} ..." >&2
  HTTP=$(curl -sS -o /tmp/vtx-claude.json -w "%{http_code}" -X POST "$URL" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d "$BODY" || true)
  if [ "$HTTP" = "200" ]; then
    echo "HTTP 200 — Vertex Claude Opus response (first 400 chars):" >&2
    head -c 400 /tmp/vtx-claude.json; echo
    exit 0
  fi
  echo "HTTP ${HTTP}" >&2
  head -c 500 /tmp/vtx-claude.json 2>/dev/null; echo >&2
done

echo "All regions failed. Check: Vertex AI API enabled, Claude partner model enabled for project, IAM (roles/vertexai.user), and model id (${MODEL_ID})." >&2
exit 1
