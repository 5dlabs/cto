#!/usr/bin/env bash
# =============================================================================
# Setup - Configure Skills and Tools from cto-config.json
# =============================================================================
#
# Mirrors the controller's configuration by extracting skills and tools from
# cto-config.json for a given agent and job type.
#
# Usage:
#   ./setup.sh [agent] [job_type]
#   ./setup.sh rex coder      # Default
#   ./setup.sh morgan intake
#   ./setup.sh blaze coder
#
# This runs:
#   1. setup-skills.sh - Copies skill SKILL.md files to config/flat-skills/
#   2. setup-tools.sh  - Generates client-config-{agent}.json for MCP tool filtering
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

AGENT="${1:-rex}"
JOB_TYPE="${2:-coder}"

echo "=============================================="
echo "Setting up Docker Compose environment"
echo "  Agent:    ${AGENT}"
echo "  Job Type: ${JOB_TYPE}"
echo "=============================================="
echo ""

# Setup skills
echo "--- Setting up skills ---"
"${SCRIPT_DIR}/setup-skills.sh" "${AGENT}" "${JOB_TYPE}"
echo ""

# Setup tools
echo "--- Setting up tools ---"
"${SCRIPT_DIR}/setup-tools.sh" "${AGENT}"
echo ""

echo "=============================================="
echo "Setup complete!"
echo ""
echo "To run the tests:"
echo "  export LINEAR_OAUTH_TOKEN=\$(op read \"op://Automation/Linear Morgan OAuth/developer_token\")"
echo "  export LINEAR_ISSUE_IDENTIFIER=CTOPA-XXXX"
echo "  docker compose up claude claude-sidecar"
echo "=============================================="
