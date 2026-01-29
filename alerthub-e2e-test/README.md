# AlertHub E2E Intake Test

This directory contains the E2E test setup for validating the full CTO intake workflow using the AlertHub PRD.

## Quick Start

```bash
# 1. Ensure services are running
./scripts/launchd-setup.sh status

# 2. Create TMUX monitoring session
./scripts/e2e-tmux-session.sh

# 3. In another terminal, attach to TMUX
tmux attach -t e2e-intake-test

# 4. Launch the swarm (from top pane in TMUX)
./scripts/launch-e2e-swarm.sh
```

## Test Coverage

This E2E test validates the complete intake workflow using AlertHub, which covers all 7 CTO platform agents:

| Agent | Component | Technology |
|-------|-----------|------------|
| Rex | Notification Router | Rust/Axum |
| Nova | Integration Service | Bun/Elysia + Effect |
| Grizz | Admin API | Go/gRPC |
| Blaze | Web Console | Next.js + Effect |
| Tap | Mobile App | Expo |
| Spark | Desktop Client | Electron |
| Bolt | Infrastructure | PostgreSQL, Redis, Kafka, MongoDB, RabbitMQ |

## Test Agents

The swarm uses 4 specialized agents:

1. **Intake Validator** - Validates task generation and workflow completion
2. **Tool Validator** - Verifies MCP tools are available and used correctly
3. **Infrastructure Monitor** - Monitors service health
4. **Linear Verifier** - Validates Linear posting and takes screenshots

## Files in This Directory

| File | Purpose |
|------|---------|
| `prd.md` | AlertHub PRD (test input) |
| `architecture.md` | System architecture (test input) |
| `e2e-config.env` | Environment configuration template |
| `swarm-prompt.md` | Coordinator prompt documentation |
| `progress.jsonl` | Workflow step progress (created during test) |
| `claude-stream.jsonl` | CLI stream output (created during test) |
| `tasks.json` | Generated tasks (created during test) |

## TMUX Session Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│ Pane 0: Swarm Coordinator (claudesp team lead output)                │
├────────────────────────────────────┬─────────────────────────────────┤
│ Pane 1: Intake Progress            │ Pane 2: CLI Stream Output       │
│ (progress.jsonl tail)              │ (claude-stream.jsonl tail)      │
├────────────────────────────────────┼─────────────────────────────────┤
│ Pane 3: Service Logs               │ Pane 4: Linear Sidecar Logs     │
│ (pm-server + controller)           │ (status-sync output)            │
└────────────────────────────────────┴─────────────────────────────────┘
```

## Success Criteria

The E2E test passes if:

- [ ] All 4 workflow steps complete (Parse PRD, Complexity, Expand, Documentation)
- [ ] Tasks generated for all 7 AlertHub components
- [ ] Tasks have complexity scores and subtasks
- [ ] MCP tools (Context7, OctoCode) are used during research
- [ ] No excessive JSON parsing retries
- [ ] Services remain healthy throughout
- [ ] Linear issue CTOPA-2608 shows plan and activities (if configured)

## Troubleshooting

### Services not running
```bash
./scripts/launchd-setup.sh install
./scripts/launchd-setup.sh status
```

### TMUX session already exists
```bash
tmux kill-session -t e2e-intake-test
./scripts/e2e-tmux-session.sh
```

### Linear credentials not set
Copy `e2e-config.env` to `.env` and fill in `LINEAR_API_KEY` and `LINEAR_TEAM_ID`.

## Skills Used

The test agents use these skills (in `~/.claude/skills/`):
- `intake-e2e-validation.md`
- `cli-tool-filtering.md`
- `infrastructure-monitoring.md`
- `linear-visual-verification.md`
