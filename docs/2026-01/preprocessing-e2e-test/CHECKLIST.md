# Implementation Checklist

## ✅ Completed

- [x] `loop.sh` - Ralph loop with claudesp-minimax + failback
- [x] `ralph-coordination.json` - 9 agents + failback state + milestones
- [x] `agents.json` - 9 agent configurations
- [x] `swarm-coordinator.md` - Coordinator prompt with all agents
- [x] `agents/oauth.md` - OAuth validation agent
- [x] `agents/environment.md` - Service health agent
- [x] `agents/intake-mcp.md` - MCP intake agent
- [x] `agents/tools-validation.md` - Tools config agent
- [x] `agents/linear-sync.md` - Linear sync agent
- [x] `agents/linear-update.md` - Linear update agent
- [x] `agents/parity.md` - Feature parity agent
- [x] `agents/critic-observer.md` - Multi-model critic agent
- [x] `agents/failback.md` - Failback monitoring agent
- [x] `issues/*.md` - 9 issue log files
- [x] `test-data/*.md` - PRD, architecture, research, resources
- [x] `tmux-session.sh` - 9-pane 3x3 layout
- [x] `README.md` - Setup and troubleshooting

## ✅ Features Agent Implementation (2026-01-28)

- [x] Create `features.md` - Feature backlog document with ACP as first feature
- [x] Create `agents/features.md` - Features implementation agent
- [x] Create `issues/issues-features.md` - Features issue log
- [x] Add `features-agent` to `ralph-coordination.json`
- [x] Add `features-agent` to `agents.json`
- [x] Update `swarm-coordinator.md` with new sequence (remediate → features → test)
- [x] Update `tmux-session.sh` for 10 panes (3x4 grid)
- [x] Update `loop.sh` - Remove MAX_ITERATIONS (infinite loop)
- [x] Add first feature to backlog: **Agent Client Protocol (ACP) for inter-agent communication**

## ❌ Outstanding

- [ ] ACP integration - Use Agent Client Protocol for inter-agent communication
- [ ] Full loop execution testing with MiniMax
- [ ] Failback trigger testing when MiniMax fails
- [ ] Linear webhook flow end-to-end
- [ ] Multi-model critic/validator testing
- [ ] Task generation and sync testing
- [ ] Bidirectional Linear updates testing

## 🔬 Needs Testing

- [ ] Full loop execution with MiniMax
- [ ] Failback trigger when MiniMax fails
- [ ] Linear webhook flow end-to-end
- [ ] Multi-model critic/validator
- [ ] Task generation and sync
- [ ] Bidirectional Linear updates

## 📝 Notes

- Command: `claudesp-minimax --dangerously-skip-permissions --permission-mode delegate --agents "$(cat agents.json)" --add-dir "$(pwd)" --verbose "$(cat swarm-coordinator.md)"`
- Failback uses `claude` instead of `claudesp-minimax`
- Plan file (`PLAN.md`) not modified
