#!/usr/bin/env bash
# =============================================================================
# Bolt Infrastructure Agent - Container Script
# Mirrors: templates/agents/bolt container behavior
# =============================================================================

set -euo pipefail

CLI_WORK_DIR="${CLI_WORK_DIR:-/workspace}"
STREAM_FILE="${CLI_WORK_DIR}/stream.jsonl"
MCP_CLIENT_CONFIG="${MCP_CLIENT_CONFIG:-/config/client-config.json}"
TOOLS_URL="${TOOLS_URL:-http://tools.fra.5dlabs.ai/mcp}"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Bolt Infrastructure Agent - Test Container"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "CLI_WORK_DIR: ${CLI_WORK_DIR}"
echo "STREAM_FILE: ${STREAM_FILE}"
echo "TOOLS_URL: ${TOOLS_URL}"
echo ""

# Ensure workspace directory exists
mkdir -p "${CLI_WORK_DIR}"
cd "${CLI_WORK_DIR}"

# Initialize stream file
> "${STREAM_FILE}"

# =============================================================================
# Configure MCP Server (cto-tools)
# =============================================================================
echo "--- Configuring MCP Server ---" >&2

if [ -f "${MCP_CLIENT_CONFIG}" ]; then
    echo "Using tool filter config: ${MCP_CLIENT_CONFIG}" >&2
    
    # Add MCP server to Claude config with tools-client wrapper for filtering
    claude mcp add cto-tools \
        --command tools \
        --args "${TOOLS_URL}" \
        --args "${CLI_WORK_DIR}" \
        -s local 2>&1 || echo "MCP config may already exist"
else
    echo "No tool filter config found, using direct connection" >&2
    claude mcp add cto-tools \
        --command tools \
        --args "${TOOLS_URL}" \
        --args "${CLI_WORK_DIR}" \
        -s local 2>&1 || echo "MCP config may already exist"
fi

echo "✓ MCP server configured" >&2
echo ""

# =============================================================================
# Verify MCP Server
# =============================================================================
echo "--- Verifying MCP Server ---" >&2

echo "Checking MCP server health..." >&2
claude mcp list 2>&1 || echo "MCP list check skipped"
echo ""

# =============================================================================
# Execute Claude CLI
# =============================================================================
echo "--- Executing Bolt Task with Claude CLI ---" >&2

# Read the prompt
PROMPT_FILE="${CLI_WORK_DIR}/prompt.md"
CLAUDE_MD="${CLI_WORK_DIR}/CLAUDE.md"

if [ ! -f "${PROMPT_FILE}" ]; then
    echo "ERROR: No prompt.md found at ${PROMPT_FILE}" >&2
    exit 1
fi

echo "Reading task from: ${PROMPT_FILE}" >&2
echo "Agent instructions: ${CLAUDE_MD}" >&2

# Execute Claude with the Bolt task
# --print outputs to stdout (for stream.jsonl)
# --dangerously-skip-permissions bypasses permission prompts
# --output-format stream-json outputs JSON stream format
claude \
    --print \
    --dangerously-skip-permissions \
    --output-format stream-json \
    --verbose \
    "Execute the infrastructure task defined in ${PROMPT_FILE}. 
     
You are Bolt, the infrastructure specialist. Read ${CLAUDE_MD} for your full instructions.

CRITICAL: For each subtask in /workspace/subtasks/, you MUST spawn a dedicated subagent using the Task tool. 
Do NOT implement subtasks yourself - delegate them to subagents.

Start by:
1. Reading /workspace/prompt.md for the main task
2. Reading /workspace/acceptance-criteria.md for acceptance criteria
3. Listing subtasks in /workspace/subtasks/
4. Spawning subagents for Level 0 subtasks (task-1.1 and task-1.2) in parallel

After all subtasks complete, create a pull request with all infrastructure code." \
    2>&1 | tee -a "${STREAM_FILE}"

echo ""
echo "✅ Bolt task complete" >&2
