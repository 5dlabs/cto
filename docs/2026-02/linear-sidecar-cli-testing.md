# Linear Sidecar CLI Testing Implementation

## Summary

This branch implements CLI-agnostic discovery of MCP tools and agent skills within the `linear-sink` crate, enabling rich status reporting in Linear's agent dialog.

## What Was Done

### Core Features Implemented

1. **Linear Sidecar Binary** (`crates/linear-sync/src/bin/linear-sidecar.rs`)
   - Runs alongside agent pods to sync status to Linear
   - Creates agent sessions on Linear issues
   - Tails `stream.jsonl` for CLI output
   - Parses each line using CLI-specific parsers
   - Emits activities to Linear agent dialog
   - Posts completion summary with tool/skill usage

2. **Stream Parsing Infrastructure**
   - `StreamParser` trait for CLI-agnostic parsing
   - `ParserRegistry` for registering CLI-specific parsers
   - Claude-specific parser (`parsers/claude.rs`) that extracts:
     - Init info (tools, skills, model)
     - Activities (thoughts, actions, results)
     - Tool usage tracking

3. **MCP Tools & Skills Display**
   - Shows initialization info with configured tools/skills
   - Completion summary displays:
     - ✅/⬜ indicators for used/unused tools
     - Skills list with usage status
     - Model used
     - Session statistics

4. **Docker Compose Test Environment** (`tests/cli-invocation/`)
   - `bolt` and `bolt-sidecar` services
   - Shared volumes mirroring Kubernetes setup
   - Config files mounted same as controller
   - Environment variables matching production

5. **Bolt Agent Configuration**
   - Model rotation: Haiku → Sonnet → Opus
   - `maxRetries: 3` for iteration testing
   - Skills and MCP tools configured via `cto-config.json`

6. **Test Scripts**
   - `run-claude.sh` - Container script mirroring `container.sh.hbs`
   - `acceptance-probe.sh` - Verifies acceptance criteria checkboxes
   - `run-with-retries.sh` - Orchestrates retry loop with model rotation
   - Complex prompt requiring multiple iterations

### Key Commits

- `a7a6016` - fix(linear-sidecar): correctly parse and display skills in completion summary
- `9a02c2b` - feat(linear-sidecar): show MCP tools with used/unused status indicators
- `c0085b7` - fix: agent-mention sensor + add detection module
- `9fe3557` - feat(test): add Bolt agent service to CLI integration tests
- `b6a574e` - feat(bolt): configure model rotation and fix prompt loading

## What's Outstanding

### Deferred Issues

1. **MCP Tools Usage Tracking**
   - Summary shows "0/19 used" even when tools were invoked
   - Root cause: Tool names in stream don't match expected MCP tool names
   - Deferred to bulk testing phase for better debugging

2. **Skills Usage Tracking**
   - Summary shows "0/15 used" 
   - Similar root cause to MCP tools tracking
   - Deferred to bulk testing

### Future Work

1. **Support Additional CLIs**
   - Implement parsers for: Codex, Aider, Cline, Roo, Gemini CLI, Amazon Q, GitHub Copilot
   - Each CLI needs specific stream parsing logic

2. **Acceptance Criteria Probe Loop**
   - Full integration with retry wrapper
   - Model rotation testing across multiple iterations

3. **Production Deployment**
   - Deploy sidecar alongside agent pods in Kubernetes
   - Verify behavior matches Docker Compose testing

## Files Changed

```
crates/linear-sync/
├── src/
│   ├── bin/linear-sidecar.rs    # Main sidecar binary
│   ├── parsers/
│   │   ├── mod.rs               # Parser registry
│   │   └── claude.rs            # Claude-specific parsing
│   ├── lib.rs                   # Public exports
│   └── emitter.rs               # Linear API integration

tests/cli-invocation/
├── docker-compose.yml           # Test services (bolt, bolt-sidecar)
├── scripts/
│   ├── run-claude.sh            # Container script
│   ├── acceptance-probe.sh      # Criteria verification
│   └── run-with-retries.sh      # Retry orchestrator
├── config/
│   ├── task/
│   │   ├── prompt.md            # Complex test prompt
│   │   └── acceptance-criteria.md
│   └── client-config-bolt.json  # Tool configuration

cto-config.json                  # Bolt agent: maxRetries, modelRotation
```

## Testing

```bash
# Build images
cd tests/cli-invocation
docker build -t cto-claude:local -f images/claude/Dockerfile ../..
docker build -f /tmp/Dockerfile.sidecar -t cto-linear-sidecar:local ../..

# Run test
docker compose down -v
rm -rf workspaces/bolt/*
docker compose up bolt bolt-sidecar

# Check Linear issue for agent dialog
```

## Verified Working

- ✅ Linear session creation
- ✅ Activity streaming to agent dialog
- ✅ Completion summary posting
- ✅ Skills listed in summary
- ✅ MCP tools listed in summary
- ✅ Bolt agent file creation
- ✅ Model rotation configuration
