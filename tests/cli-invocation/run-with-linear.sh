#!/usr/bin/env bash
# =============================================================================
# Run CLI with Linear Sidecar Integration
#
# Mirrors production CodeRun setup:
# - Bare bones CLI image
# - All config mounted (MCP, skills, etc.) - NOT baked into image
# - Output piped to sidecar for Linear posting
# - Runtime Linear token sourced from PM/Kubernetes, not 1Password
#
# Usage:
#   LINEAR_ISSUE_IDENTIFIER="CTOPA-123" ./tests/cli-invocation/run-with-linear.sh
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Workspace for this test run
WORKSPACE="${SCRIPT_DIR}/workspaces/claude"
mkdir -p "$WORKSPACE"

# Paths (mirror production ConfigMap mounts)
SKILLS_PATH="${PROJECT_ROOT}/templates/skills"
MCP_CONFIG="${WORKSPACE}/mcp.json"
DOCTOR_OUTPUT="${WORKSPACE}/claude-doctor.txt"

# =============================================================================
# Environment validation
# =============================================================================
if [[ -z "${LINEAR_OAUTH_TOKEN:-}" ]]; then
    PM_BASE_URL="${PM_BASE_URL:-https://pm.5dlabs.ai}"
    NAMESPACE="${NAMESPACE:-cto}"
    LINEAR_AGENT_FOR_TESTS="${LINEAR_AGENT_FOR_TESTS:-morgan}"

    curl -fsS -X POST "${PM_BASE_URL}/oauth/mint/${LINEAR_AGENT_FOR_TESTS}" >/dev/null 2>&1 || true

    LINEAR_OAUTH_TOKEN=$(kubectl get secret "linear-app-${LINEAR_AGENT_FOR_TESTS}" -n "${NAMESPACE}" \
        -o jsonpath='{.data.access_token}' 2>/dev/null | base64 -d 2>/dev/null || echo "")
    if [[ -z "$LINEAR_OAUTH_TOKEN" ]]; then
        echo "Need LINEAR_OAUTH_TOKEN. Set it or let PM mint one into linear-app-${LINEAR_AGENT_FOR_TESTS}."
        exit 1
    fi
    export LINEAR_OAUTH_TOKEN
fi

if [[ -z "${LINEAR_ISSUE_ID:-}" && -z "${LINEAR_ISSUE_IDENTIFIER:-}" ]]; then
    echo "Set LINEAR_ISSUE_ID (issue UUID) or LINEAR_ISSUE_IDENTIFIER (e.g. CTOPA-2620)."
    exit 1
fi

if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
    ANTHROPIC_API_KEY=$(op read "op://Automation/Anthropic API Key/credential" 2>/dev/null || echo "")
    if [[ -z "$ANTHROPIC_API_KEY" ]]; then
        # Fallback to cluster secret
        ANTHROPIC_API_KEY=$(kubectl get secret cto-secrets -n cto -o jsonpath='{.data.ANTHROPIC_API_KEY}' 2>/dev/null | base64 -d || echo "")
    fi
    export ANTHROPIC_API_KEY
fi

# =============================================================================
# Create MCP config (mirrors ConfigMap but uses public URL for Docker access)
# =============================================================================
cat > "$MCP_CONFIG" << 'EOF'
{
  "mcpServers": {
    "cto-tools": {
      "url": "http://tools.fra.5dlabs.ai/mcp",
      "transport": {
        "type": "sse"
      }
    }
  }
}
EOF

# =============================================================================
# Test info
# =============================================================================
ISSUE_DISPLAY="${LINEAR_ISSUE_IDENTIFIER:-${LINEAR_ISSUE_ID:-}}"
echo "═══════════════════════════════════════════════════════════════"
echo "║          CLI + Linear Sidecar Test                          ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Issue:       ${ISSUE_DISPLAY}"
echo "Skills:      ${SKILLS_PATH}"
echo "MCP Config:  ${MCP_CONFIG}"
echo "Workspace:   ${WORKSPACE}"
echo ""

# Open the Linear issue in the browser
LINEAR_WORKSPACE_SLUG="${LINEAR_WORKSPACE_SLUG:-jonathonfritz}"
if [[ -n "${LINEAR_ISSUE_IDENTIFIER:-}" ]]; then
    ISSUE_URL="https://linear.app/${LINEAR_WORKSPACE_SLUG}/issue/${LINEAR_ISSUE_IDENTIFIER}"
    echo "Opening: ${ISSUE_URL}"
    open "$ISSUE_URL" 2>/dev/null || xdg-open "$ISSUE_URL" 2>/dev/null || true
    sleep 1
fi

# =============================================================================
# Run Claude container
# Mounts mirror production CodeRun:
#   /templates/skills - skills from ConfigMap
#   /mcp-config       - MCP config from ConfigMap
#   /workspace        - working directory
# =============================================================================
PROMPT="${PROMPT:-You are testing our Linear agent dialog integration. Do these steps:

1. Create a file /workspace/hello.txt with 'Hello World'
2. List files in /workspace
3. Create /workspace/script.sh that echoes 'Test complete!'
4. Run the script
5. Summarize what you did, including which tools you used

Be thorough. This tests our agent tracking.}"

echo ""
echo "Running Claude with mounted config..."
echo ""

(
    docker run --rm -i \
        -v "${WORKSPACE}:/workspace" \
        -v "${SKILLS_PATH}:/templates/skills:ro" \
        -v "${MCP_CONFIG}:/mcp-config/mcp.json:ro" \
        -e "ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}" \
        -e "SKILLS_PATH=/templates/skills" \
        -e "CLI_WORK_DIR=/workspace" \
        --network host \
        cto-claude:local \
        sh -c '
            # Run claude doctor first and save output (for sidecar to read)
            claude doctor > /workspace/claude-doctor.txt 2>&1 || true
            
            # Run Claude with MCP config
            echo "n" | claude --print --output-format stream-json --verbose \
                --mcp-config /mcp-config/mcp.json --strict-mcp-config \
                --dangerously-skip-permissions \
                "'"${PROMPT}"'"
        ' 2>&1
) | tee "${WORKSPACE}/stream.jsonl" | docker run --rm -i \
    -v "${WORKSPACE}:/workspace:ro" \
    -e "LINEAR_OAUTH_TOKEN=${LINEAR_OAUTH_TOKEN}" \
    -e "LINEAR_ISSUE_ID=${LINEAR_ISSUE_ID:-}" \
    -e "LINEAR_ISSUE_IDENTIFIER=${LINEAR_ISSUE_IDENTIFIER:-}" \
    -e "CLI_TYPE=claude" \
    -e "DOCTOR_OUTPUT=/workspace/claude-doctor.txt" \
    -e "RUST_LOG=info" \
    cto-linear-sidecar:local

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Done. Check the issue's agent dialog in Linear."
echo "═══════════════════════════════════════════════════════════════"
