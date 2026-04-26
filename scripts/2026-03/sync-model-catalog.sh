#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PROVIDERS="${MODEL_SYNC_PROVIDERS:-anthropic,openai}"
NORMALIZED_OUT="${MODEL_SYNC_NORMALIZED_OUT:-infra/model-catalog/normalized-model-catalog.json}"
OPENCLAW_OUT="${MODEL_SYNC_OPENCLAW_OUT:-infra/charts/openclaw-agent/files/model-catalog.generated.json}"
WEB_OUT="${MODEL_SYNC_WEB_OUT:-apps/web/src/generated/model-catalog.json}"

echo "Syncing model catalog for providers: ${PROVIDERS}"
python3 scripts/2026-03/normalize-model-catalog.py \
  --providers "${PROVIDERS}" \
  --normalized-out "${NORMALIZED_OUT}" \
  --openclaw-out "${OPENCLAW_OUT}" \
  --web-out "${WEB_OUT}"

echo "Model catalog sync complete."
