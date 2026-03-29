#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE_DIR="$ROOT/infra/observability/local"

# Ensure log directory exists
mkdir -p "$ROOT/.intake/logs"

echo "Starting local observability stack..."
docker compose -f "$COMPOSE_DIR/docker-compose.yml" up -d

echo ""
echo "✓ Observability stack is running"
echo "  Grafana:  http://localhost:3001"
echo "  Loki:     http://localhost:3100"
echo ""
echo "  Log files: $ROOT/.intake/logs/"
echo "  Dashboards auto-refresh from: $COMPOSE_DIR/dashboards/"
