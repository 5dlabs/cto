#!/usr/bin/env bash
# =============================================================================
# Setup Tools - Mirror Controller's Tool Resolution
# =============================================================================
#
# Generates client-config-{agent}.json from cto-config.json, mirroring how
# the controller generates client-config.json for MCP tool filtering.
#
# Usage:
#   ./setup-tools.sh [agent]
#   ./setup-tools.sh rex       # Default
#   ./setup-tools.sh blaze
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTO_ROOT="${SCRIPT_DIR}/../.."
CTO_CONFIG="${CTO_ROOT}/cto-config.json"
CONFIG_DIR="${SCRIPT_DIR}/config"

AGENT="${1:-rex}"

echo "Setting up tools config for agent '${AGENT}'"
echo "  Config: ${CTO_CONFIG}"
echo "  Output: ${CONFIG_DIR}/client-config-${AGENT}.json"

if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed"
    exit 1
fi

# Extract remote tools for the agent
REMOTE_TOOLS=$(jq -r ".agents.${AGENT}.tools.remote // []" "${CTO_CONFIG}" 2>/dev/null)

if [ "${REMOTE_TOOLS}" == "[]" ] || [ -z "${REMOTE_TOOLS}" ]; then
    echo "Warning: No remote tools found for agent '${AGENT}' in cto-config.json"
fi

# Generate client-config.json
cat > "${CONFIG_DIR}/client-config-${AGENT}.json" << EOF
{
  "remoteTools": ${REMOTE_TOOLS},
  "localServers": {}
}
EOF

echo ""
echo "Generated ${CONFIG_DIR}/client-config-${AGENT}.json:"
cat "${CONFIG_DIR}/client-config-${AGENT}.json"
