#!/bin/bash
# Launch E2E Intake Test Swarm
#
# This script launches claudesp in delegate mode to coordinate the E2E test
# swarm with 4 specialized agents.
#
# Prerequisites:
#   1. Services running: ./scripts/launchd-setup.sh status
#   2. Linear credentials set (optional, for posting to CTOPA-2608)
#   3. TMUX session created: ./scripts/e2e-tmux-session.sh
#
# Usage:
#   ./scripts/launch-e2e-swarm.sh           # Interactive mode
#   ./scripts/launch-e2e-swarm.sh --print   # Non-interactive mode

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE="$(dirname "$SCRIPT_DIR")"
WORK_DIR="${WORKSPACE}/alerthub-e2e-test"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== E2E Intake Test Swarm Launcher ===${NC}"
echo ""

# Check services
echo -e "${YELLOW}Checking services...${NC}"
if ! curl -sf http://localhost:8081/health > /dev/null 2>&1; then
    echo -e "${RED}PM Server not healthy. Start with: ./scripts/launchd-setup.sh install${NC}"
    exit 1
fi
if ! curl -sf http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${RED}Controller not healthy. Start with: ./scripts/launchd-setup.sh install${NC}"
    exit 1
fi
echo -e "${GREEN}Services healthy${NC}"
echo ""

# Change to work directory
cd "$WORK_DIR"

# Create output files if they don't exist
touch progress.jsonl claude-stream.jsonl 2>/dev/null || true

# Define agents for the swarm (6 agents total)
AGENTS_JSON='{
  "intake-validator": {
    "description": "Runs intake workflow - TASK GENERATION IS PRIORITY #1",
    "prompt": "You are the Intake Task Agent. TASK GENERATION IS YOUR #1 PRIORITY. Run the intake binary directly: ./target/release/intake intake --prd ./alerthub-e2e-test/prd.md --architecture ./alerthub-e2e-test/architecture.md --use-cli -o ./alerthub-e2e-test/.tasks -n 50. If CLI mode fails (JSON parsing errors), FALLBACK to API mode (remove --use-cli). Success = tasks.json has 15+ tasks covering Rex, Nova, Grizz, Blaze, Tap, Spark, Bolt."
  },
  "tool-validator": {
    "description": "Validates CLI tool availability",
    "prompt": "You are the CLI Tool Filtering Agent. Your job is to verify MCP tools are correctly available. Check claude-stream.jsonl for system events listing tools, verify Context7 and OctoCode tools are available, and report any tool errors."
  },
  "infra-monitor": {
    "description": "Monitors infrastructure services",
    "prompt": "You are the Infrastructure Agent. Your job is to monitor services and handle restarts. Check health endpoints at localhost:8081 and localhost:8080, monitor logs at /tmp/cto-launchd/, and restart services if needed. All credentials are in 1Password - use the op CLI to fetch them."
  },
  "linear-verifier": {
    "description": "Verifies Linear posting and takes screenshots",
    "prompt": "You are the Linear Verification Agent. Your job is to verify progress is posted to Linear. Navigate to Linear issue CTOPA-2608, take screenshots of the agent dialog, verify plan and activities are visible."
  },
  "issue-remediator": {
    "description": "Fixes BLOCKING issues from e2e-test-issues.md",
    "prompt": "You are the Issue Remediation Agent. Read /Users/jonathonfritz/cto-e2e-testing/alerthub-e2e-test/e2e-test-issues.md and fix BLOCKING issues. Priority 1: CLI JSON parsing (study Taskmaster-AI at https://github.com/eyaltoledano/claude-task-master). Priority 2: MCP local mode. Clone taskmaster: gh repo clone eyaltoledano/claude-task-master /tmp/taskmaster, then study src/ai/ and mcp-server/src/tools/parse.js for JSON handling patterns."
  },
  "json-debugger": {
    "description": "Debugs JSON parsing - studies Taskmaster-AI patterns",
    "prompt": "You are the JSON Debug Agent. Clone Taskmaster-AI: gh repo clone eyaltoledano/claude-task-master /tmp/taskmaster. Study their JSON handling in src/ai/anthropic.js and src/core/task-master-core.js. Compare to our crates/intake/src/ai/cli_provider.rs and crates/intake/src/ai/provider.rs. Report differences and implement fixes."
  }
}'

# Coordinator prompt - Read the full swarm prompt from file
SWARM_PROMPT_FILE="${WORK_DIR}/swarm-prompt.md"

if [[ -f "$SWARM_PROMPT_FILE" ]]; then
    COORDINATOR_PROMPT=$(cat "$SWARM_PROMPT_FILE")
else
    COORDINATOR_PROMPT="You are coordinating an E2E test of the CTO intake workflow.

CRITICAL: TASK GENERATION IS PRIORITY #1

Your team (6 agents):
- intake-validator: Runs intake workflow (PRIORITY #1)
- tool-validator: Validates MCP tool availability
- infra-monitor: Monitors services
- linear-verifier: Verifies Linear posting
- issue-remediator: Fixes BLOCKING issues
- json-debugger: Studies Taskmaster-AI JSON patterns

ESCALATION STRATEGY:
1. Try CLI mode first (--use-cli flag)
2. If JSON parsing fails, study Taskmaster-AI: gh repo clone eyaltoledano/claude-task-master
3. Fall back to API mode if needed (remove --use-cli)

Your task:
1. First spawn infra-monitor to verify services
2. Spawn json-debugger and issue-remediator to study Taskmaster-AI patterns
3. Then spawn intake-validator to run the workflow
4. Spawn tool-validator and linear-verifier in parallel
5. Collect status reports from all agents
6. If intake fails, coordinate with remediator for fixes, then retry

Test inputs: ${WORK_DIR}
- prd.md - AlertHub PRD
- architecture.md - System architecture

SUCCESS = tasks.json contains 15+ tasks covering all 7 AlertHub components.
This is the ONLY metric that matters."
fi

# Launch options
PRINT_MODE=""
if [[ "${1:-}" == "--print" ]]; then
    PRINT_MODE="--print --output-format stream-json"
    echo -e "${YELLOW}Running in non-interactive mode${NC}"
fi

echo -e "${GREEN}Launching claudesp swarm coordinator...${NC}"
echo -e "Work directory: ${WORK_DIR}"
echo ""
echo -e "${YELLOW}Attach to TMUX session for monitoring:${NC}"
echo "  tmux attach -t e2e-intake-test"
echo ""

# Launch claudesp with delegate mode and custom agents
# Use --dangerously-skip-permissions to run fully unattended
claudesp \
    --dangerously-skip-permissions \
    --permission-mode delegate \
    --agents "${AGENTS_JSON}" \
    --add-dir "${WORKSPACE}" \
    --verbose \
    ${PRINT_MODE} \
    "${COORDINATOR_PROMPT}"
