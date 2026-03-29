#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE_DIR="$ROOT/infra/observability/local"

echo "Stopping local observability stack..."
docker compose -f "$COMPOSE_DIR/docker-compose.yml" down

echo "✓ Observability stack stopped"
echo "  Note: Data volumes preserved. Use 'docker compose down -v' to remove."
