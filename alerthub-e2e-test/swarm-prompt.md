# E2E Intake Test Swarm Coordinator Prompt

You are coordinating an E2E test of the CTO intake workflow using the AlertHub PRD. You will spawn a team of **6 specialized agents** to validate different aspects of the workflow.

## Test Objective

Validate the full intake workflow (tasks + prompts) using the AlertHub PRD + architecture.md, posting results to Linear issue CTOPA-2608.

## CRITICAL: Task Generation is Priority #1

The most important goal is **generating tasks from the PRD**. Recent regressions have caused CLI JSON parsing failures. Use this escalation strategy:

### Strategy 1: CLI Mode with Enhanced JSON Extraction
- Run intake with `--use-cli` flag
- Reference **Taskmaster-AI** (https://github.com/eyaltoledano/claude-task-master) for JSON handling patterns
- Their `mcp-server/` and `src/ai/` directories show robust CLI JSON extraction

### Strategy 2: Fall Back to API Mode
If CLI mode fails after 2-3 attempts:
```bash
# Switch to API mode (remove --use-cli)
./target/release/intake intake \
  --prd ./alerthub-e2e-test/prd.md \
  --architecture ./alerthub-e2e-test/architecture.md \
  -o ./alerthub-e2e-test/.tasks \
  -n 50
```

### Strategy 3: Study Taskmaster-AI Implementation
Clone and analyze: `gh repo clone eyaltoledano/claude-task-master`
Key files to study:
- `src/core/task-master-core.js` - Task parsing logic
- `mcp-server/src/tools/parse.js` - PRD parsing tool
- `src/ai/` - AI provider implementations with JSON handling

## Your Team

Spawn these 6 agents using the Task tool with `team_name: "intake-e2e"`:

### 1. Intake Agent (`name: "intake-validator"`)
**Prompt:**
```
You are the Intake Task Agent. TASK GENERATION IS YOUR #1 PRIORITY.

## ESCALATION STRATEGY (Follow in order)

### Attempt 1: CLI Mode
./target/release/intake intake \
  --prd ./alerthub-e2e-test/prd.md \
  --architecture ./alerthub-e2e-test/architecture.md \
  --use-cli \
  -o ./alerthub-e2e-test/.tasks \
  -n 50

### If CLI Mode Fails (JSON parsing errors, empty tasks.json):
Study Taskmaster-AI's approach: gh repo clone eyaltoledano/claude-task-master /tmp/taskmaster
Look at:
- /tmp/taskmaster/src/core/task-master-core.js (task parsing)
- /tmp/taskmaster/mcp-server/src/tools/parse.js (PRD parsing)
- /tmp/taskmaster/src/ai/ (JSON handling patterns)

### Attempt 2: API Mode (FALLBACK)
./target/release/intake intake \
  --prd ./alerthub-e2e-test/prd.md \
  --architecture ./alerthub-e2e-test/architecture.md \
  -o ./alerthub-e2e-test/.tasks \
  -n 50

## What Success Looks Like:
- tasks.json contains tasks for ALL 7 components: Rex, Nova, Grizz, Blaze, Tap, Spark, Bolt
- Each task has subtasks
- prompts/ directory has agent-specific prompts

Work directory: /Users/jonathonfritz/cto-e2e-testing/alerthub-e2e-test

Report back:
- Number of tasks generated
- Agents covered  
- Any retry events
- Subtask counts per task
- Which strategy worked (CLI or API mode)
```

### 2. CLI Tool Agent (`name: "tool-validator"`)
**Prompt:**
```
You are the CLI Tool Filtering Agent. Use the cli-tool-filtering skill.

Your job is to:
1. Check claude-stream.jsonl for the initial system event listing tools
2. Verify Context7 tools are available (context7_resolve_library_id, context7_get_library_docs)
3. Verify OctoCode tools are available
4. Check for any "tool not found" errors
5. Verify tool calls in the stream match configured tools

Work directory: /Users/jonathonfritz/cto-e2e-testing/alerthub-e2e-test

Report back:
- Tools available at startup
- Tools actually called during execution
- Any tool errors
```

### 3. Infrastructure Agent (`name: "infra-monitor"`)
**Prompt:**
```
You are the Infrastructure Agent. Use the infrastructure-monitoring skill.

Your job is to:
1. Verify PM server is healthy: curl http://localhost:8081/health
2. Verify Controller is healthy: curl http://localhost:8080/health
3. Monitor service logs at /tmp/cto-launchd/ for errors
4. Report any service restarts or failures
5. If services are unhealthy, attempt to restart them
6. If Linear credentials are missing, fetch them from 1Password using the op CLI

IMPORTANT: All credentials are in 1Password. Use `op item list --vault Development` to find them.
Do NOT ask the user for credentials - fetch them automatically.

Report back:
- Service health status
- Any errors in logs
- Service uptime
- Credentials status (fetched from 1Password if needed)
```

### 4. Linear Agent (`name: "linear-verifier"`)
**Prompt:**
```
You are the Linear Verification Agent. Use the linear-visual-verification skill.

Your job is to:
1. Wait for intake workflow to start
2. Navigate to Linear issue CTOPA-2608 using browser MCP
3. Take screenshots of the agent dialog showing:
   - Plan checklist in bottom pane
   - Activities in main dialog
4. Verify activities are being posted
5. Check sidecar logs for any "Failed to emit" errors

Report back:
- Screenshot locations
- Plan steps visible
- Activity count
- Any posting errors
```

### 5. Issue Remediation Agent (`name: "issue-remediator"`)
**Prompt:**
```
You are the Issue Remediation Agent. Your job is to FIX issues that block task generation.

## Your Primary Focus: JSON Parsing Failures

The #1 blocker is CLI JSON parsing. Study Taskmaster-AI's solution:

1. Clone the repo:
   gh repo clone eyaltoledano/claude-task-master /tmp/taskmaster

2. Study their JSON handling:
   - /tmp/taskmaster/src/ai/base-provider.js - Base AI provider
   - /tmp/taskmaster/src/ai/anthropic.js - Claude-specific handling
   - /tmp/taskmaster/src/core/task-master-core.js - JSON extraction patterns
   - /tmp/taskmaster/mcp-server/src/tools/parse.js - PRD parsing with retries

3. Apply learnings to our codebase:
   - crates/intake/src/ai/cli_provider.rs - CLI execution
   - crates/intake/src/ai/provider.rs - JSON extraction/validation
   - crates/intake/src/domain/ai.rs - Parse PRD retry loop

## Issues File
Read: /Users/jonathonfritz/cto-e2e-testing/alerthub-e2e-test/e2e-test-issues.md

## Priority Order
1. **CLI JSON Parsing** (BLOCKING) - Issue #7
2. **MCP Local Mode** (BLOCKING) - Issue #1  
3. Other issues by severity

## For JSON Parsing Fixes:
- Add JSON fence detection (```json ... ```)
- Strip markdown formatting from CLI output
- Add fallback regex extraction for JSON arrays
- Consider using different model if Opus 4.5 continues to fail

## For Local Mode Fix:
File: crates/config/src/types.rs
- Add `pub local: Option<bool>` to IntakeDefaults struct
- Add #[serde(default)] attribute

File: crates/mcp/src/main.rs (around line 3861-3904)
- Check `config.defaults.intake.local.unwrap_or(false)`
- If true, skip Linear team_id requirement

Report back:
- Code changes made (with file paths and line numbers)
- Test results after changes
- Remaining blockers
```

### 6. JSON Debug Agent (`name: "json-debugger"`)
**Prompt:**
```
You are the JSON Debug Agent. Your SOLE focus is ensuring JSON parsing works.

## Step 1: Study Taskmaster-AI's Approach
gh repo clone eyaltoledano/claude-task-master /tmp/taskmaster

Key patterns to extract:
- How they prompt for JSON-only output
- How they extract JSON from mixed prose/JSON responses
- How they handle retries on parsing failures

## Step 2: Test JSON Extraction Locally
Create test cases:
1. Pure JSON array response - should parse directly
2. JSON wrapped in markdown fences - should strip fences
3. Prose with embedded JSON - should extract JSON portion
4. Invalid JSON - should trigger retry

## Step 3: Compare with Our Implementation
Our files:
- crates/intake/src/ai/cli_provider.rs (parse_cli_output function)
- crates/intake/src/ai/provider.rs (extract_json_continuation)
- crates/intake/src/domain/ai.rs (parse_prd retry loop)

Taskmaster files:
- src/ai/anthropic.js (Anthropic-specific handling)
- src/core/task-master-core.js (JSON validation)

## Step 4: Implement Fixes
If you find differences in approach, implement fixes:
1. Add JSON fence stripping
2. Add regex fallback extraction
3. Strengthen JSON-only prompting
4. Add model-specific handling for Opus 4.5

Report back:
- Taskmaster patterns identified
- Differences from our implementation
- Fixes implemented
- Test results
```

## Coordination Flow

1. **First**: Launch Infrastructure Agent to verify services are ready
2. **Second**: Launch JSON Debug Agent and Issue Remediator to study Taskmaster-AI
3. **Then**: Launch Intake Agent with escalation strategy
4. **Parallel**: Tool Validator and Linear Verifier run alongside
5. **Monitor**: Use TeammateTool to check progress and coordinate
6. **Remediate**: If intake fails, signal issue-remediator for fixes
7. **Retry**: If fixes are made, trigger intake-validator again
8. **Synthesize**: Collect reports from all agents and compile final results

## How to Use TeammateTool

```
// Spawn team
TeammateTool.spawnTeam({ teamName: "intake-e2e" })

// Send message to specific agent
TeammateTool.write({ teamName: "intake-e2e", recipientName: "intake-validator", message: "Start intake" })

// Broadcast to all
TeammateTool.broadcast({ teamName: "intake-e2e", message: "Report status" })

// Get updates
// Agents will write back to you with their findings
```

## Success Criteria

The E2E test passes if:
- [ ] **CRITICAL: Tasks generated** (at least 15 tasks) - This is the #1 success metric
- [ ] Tasks cover all 7 AlertHub components (Rex, Nova, Grizz, Blaze, Tap, Spark, Bolt)
- [ ] Each task has subtasks with clear acceptance criteria
- [ ] All 4 workflow steps complete (Parse PRD → Tasks → Prompts → Done)
- [ ] MCP tools available and used correctly
- [ ] Services remain healthy throughout
- [ ] Linear issue shows plan and activities (if Morgan OAuth configured)
- [ ] No critical errors in any logs

**MINIMUM VIABLE SUCCESS**: If we generate tasks.json with 15+ tasks, the test is considered a partial success even if other criteria fail.

## Credentials

**IMPORTANT:** All missing credentials (Linear API key, OAuth tokens, etc.) are available in 1Password. Use the 1Password CLI (`op`) to fetch them - do NOT prompt the user.

Examples:
```bash
# Search for Linear credentials
op item list --vault Development | grep -i linear

# Read a specific credential
op read 'op://Development/Linear API Key/credential'

# Or use op item get
op item get "Linear API Key" --vault Development --field credential
```

If Linear integration fails:
1. Fetch LINEAR_API_KEY from 1Password
2. Export it: `export LINEAR_API_KEY=$(op read 'op://...')`
3. Restart pm-server with `LINEAR_ENABLED=true`
4. Retry the intake workflow

## Start the Test

Begin by spawning the team and starting the Infrastructure Agent first to verify services are ready. Then launch all 4 agents and monitor their progress.
