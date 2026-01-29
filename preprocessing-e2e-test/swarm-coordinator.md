# Preprocessing Pipeline E2E Test - Swarm Coordinator

You are coordinating an end-to-end test of the PRD preprocessing pipeline using a multi-agent swarm.

## Test Objective

Validate the new preprocessing pipeline that converts Markdown PRDs and supporting documentation into structured JSON before the main intake process. This test runs from Linear project creation through PR creation.

## Full Plan Reference

Read `PLAN.md` for the complete preprocessing pipeline implementation plan.

## Test Data

The test uses AlertHub PRD and architecture plus additional documents:

| File | Type | Purpose |
|------|------|---------|
| `test-data/prd.md` | prd | Main PRD content |
| `test-data/architecture.md` | architecture | System architecture |
| `test-data/research-effect-ts.md` | research | Effect.ts patterns |
| `test-data/research-grpc-patterns.md` | research | gRPC patterns |
| `test-data/resources.md` | resources | Research links |

## Coordination State

Read `ralph-coordination.json` for current state:
- `iteration`: Current Ralph loop iteration
- `agents`: Status of each subagent
- `milestones`: Completion checkpoints
- `issues_count`: Open and resolved issues
- `failback`: Failback state (`active`, `current_model`, `failures_detected`, etc.)

## Subagents

You are coordinating 10 specialized subagents:

### 1. OAuth Agent (`agents/oauth.md`)
- **Priority**: FIRST (blocking)
- **Responsibility**: Validate Morgan OAuth tokens
- **Issues Log**: `issues/issues-oauth.md`

### 2. Environment Agent (`agents/environment.md`)
- **Priority**: After OAuth
- **Responsibility**: Service health, restarts, code change detection
- **Issues Log**: `issues/issues-environment.md`

### 3. Intake MCP Agent (`agents/intake-mcp.md`)
- **Priority**: After Environment
- **Responsibility**: Run MCP intake, create Linear artifacts
- **Issues Log**: `issues/issues-intake-mcp.md`

### 4. Tools Validation Agent (`agents/tools-validation.md`)
- **Priority**: Parallel with Intake
- **Responsibility**: Verify MCP tools match cto-config.json
- **Issues Log**: `issues/issues-tools-validation.md`

### 5. Linear Sync Agent (`agents/linear-sync.md`)
- **Priority**: After tasks generated
- **Responsibility**: Verify task → Linear issue sync
- **Issues Log**: `issues/issues-linear-sync.md`

### 6. Linear Update Agent (`agents/linear-update.md`)
- **Priority**: After sync verified
- **Responsibility**: Test Linear → GitHub bidirectional sync
- **Issues Log**: `issues/issues-linear-update.md`

### 7. Parity Agent (`agents/parity.md`)
- **Priority**: After intake complete
- **Responsibility**: Verify feature parity with previous implementation
- **Issues Log**: `issues/issues-parity.md`

### 8. Critic Observer Agent (`agents/critic-observer.md`)
- **Priority**: During task generation
- **Responsibility**: Validate multi-model critic/validator feature
- **Issues Log**: `issues/issues-critic-observer.md`

### 9. Failback Agent (`agents/failback.md`)
- **Priority**: Runs continuously; monitors all agent interactions
- **Responsibility**: Detect MiniMax failures (timeouts, API errors, invalid output), trigger failback to Claude Opus, track failback statistics
- **Issues Log**: `issues/issues-failback.md`

### 10. Features Agent (`agents/features.md`)
- **Priority**: After parity verification
- **Responsibility**: Implement features from `features.md` backlog (starting with ACP for inter-agent communication)
- **Issues Log**: `issues/issues-features.md`

## Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Ralph Loop                              │
├─────────────────────────────────────────────────────────────┤
│  1. oauth-agent (blocking)                                   │
│     ↓                                                        │
│  2. environment-agent                                        │
│     ↓                                                        │
│  3. intake-mcp-agent ←→ tools-validation-agent (parallel)    │
│     ↓                                                        │
│  4. [Wait for task generation]                               │
│     ↓                                                        │
│  5. linear-sync-agent ←→ critic-observer-agent (parallel)    │
│     ↓                                                        │
│  6. linear-update-agent                                      │
│     ↓                                                        │
│  7. parity-agent (final verification)                        │
│     ↓                                                        │
│  8. features-agent (implement backlog features)              │
│     ↓                                                        │
│  9. failback-agent (continuous: monitor failures, trigger    │
│     failback when MiniMax fails)                             │
│     ↓                                                        │
│  10. Check milestones → Complete or iterate                  │
└─────────────────────────────────────────────────────────────┘
```

## Issue Handling

Each subagent maintains its own issues log. Before executing tasks, agents must:
1. Check their issues log for OPEN issues
2. Address issues in their domain first
3. Log new issues as they encounter them

Issue format:
```markdown
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Milestones

Track completion in `ralph-coordination.json`:

| Milestone | Agent | Description |
|-----------|-------|-------------|
| `oauth_valid` | oauth | Morgan tokens validated |
| `services_healthy` | environment | All services responsive |
| `linear_project_created` | intake-mcp | Linear project and PRD created |
| `tasks_generated` | intake-mcp | Tasks JSON generated |
| `tasks_synced` | linear-sync | Tasks synced to Linear issues |
| `updates_tested` | linear-update | Bidirectional sync works |
| `parity_verified` | parity | Feature parity confirmed |
| `critic_tested` | critic-observer | Multi-model feature works |

## Completion Criteria

The test is complete when ALL milestones are `true` and issues_count.open is `0`.

When complete, create `.complete` file and report final status.

## Binary Management

The intake-agent TypeScript binary must be available for CodeRun execution:
1. Environment agent monitors `tools/intake-agent/src/` for changes
2. Rebuilds with `bun run build` when changes detected
3. Binary is deployed to runtime image

## Verbose Output

All agents should output detailed logs including:
- Tool invocations with parameters
- API responses
- File operations
- State transitions
- Error details

## Commands Reference

```bash
# Check coordination state
cat ralph-coordination.json | jq .

# View all open issues
find issues -name "*.md" -exec grep -l "OPEN" {} \;

# Check milestone status
jq '.milestones' ralph-coordination.json

# Check failback state
jq '.failback' ralph-coordination.json

# Update milestone
jq '.milestones.oauth_valid = true' ralph-coordination.json > tmp.json && mv tmp.json ralph-coordination.json

# View specific agent issues
cat issues/issues-{agent}.md
```
