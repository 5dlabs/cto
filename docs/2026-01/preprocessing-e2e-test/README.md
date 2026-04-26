# Preprocessing E2E Test - Complete Plan & Status

## Quick Reference

**Run Command:**
```bash
claudesp-minimax \
  --dangerously-skip-permissions \
  --permission-mode delegate \
  --agents "$(cat agents.json)" \
  --add-dir "$(pwd)" \
  --verbose \
  "$(cat swarm-coordinator.md)"
```

**Install:**
```bash
npx @realmikekelly/claude-sneakpeek quick \
  --provider minimax \
  --api-key "$MINIMAX_API_KEY" \
  --name claudesp-minimax
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Ralph Loop (loop.sh)                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │  1. Check failback.active → use claude or claudesp-minimax       │  │
│  │  2. Run swarm with --agents agents.json                          │  │
│  │  3. Check milestones → complete or iterate                       │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    claudesp-minimax Swarm (MiniMax M2.1)                │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     swarm-coordinator.md                         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│  ┌─────────┬─────────┬─────────┬─────────┬─────────┐                   │
│  │ oauth   │ environ │ intake  │ tools   │ linear  │                   │
│  │ agent   │ agent   │ mcp     │ valid   │ sync    │                   │
│  └────┬────┴────┬────┴────┬────┴────┬────┴────┬────┘                   │
│  ┌────┴────┬────┴────┬────┴────┬────┴────┐                             │
│  │ linear  │ parity  │ critic  │failback │                             │
│  │ update  │ agent   │observer │ agent   │                             │
│  └─────────┴─────────┴─────────┴─────────┘                             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        Coordination & State                             │
│  ralph-coordination.json   issues/*.md   milestones   failback state   │
└─────────────────────────────────────────────────────────────────────────┘
```

## TMux Layout (3x3)

```
┌─────────────────┬─────────────────┬─────────────────┐
│  oauth          │  environment    │  intake-mcp     │
├─────────────────┼─────────────────┼─────────────────┤
│  tools-valid    │  linear-sync    │  linear-update  │
├─────────────────┼─────────────────┼─────────────────┤
│  parity         │  critic-observer│  failback       │
└─────────────────┴─────────────────┴─────────────────┘
```

---

## Related Plans

| Plan | Location | Purpose |
|------|----------|---------|
| Intake SDK Migration | `docs/intake_sdk_migration_61705da7.plan.md` | TypeScript intake with Claude SDK |
| PRD Preprocessing | `docs/prd_preprocessing_pipeline_39fc3823.plan.md` | Markdown→JSON preprocessing |
| MiniMax Swarm | This folder | E2E test orchestration |

---

## Implementation Status

### ✅ DONE

| Item | File | Notes |
|------|------|-------|
| Loop script | `loop.sh` | claudesp-minimax + failback to claude |
| Coordination state | `ralph-coordination.json` | 9 agents + failback state |
| Agent config | `agents.json` | 9 agents with priorities |
| 9 agent prompts | `agents/*.md` | Including failback.md |
| 9 issue logs | `issues/*.md` | Including issues-failback.md |
| Swarm coordinator | `swarm-coordinator.md` | References all agents + failback |
| TMux layout | `tmux-session.sh` | 3x3 grid, 9 panes |
| Test data | `test-data/*.md` | PRD, architecture, research, resources |
| README | `README.md` | Setup + troubleshooting |

### ❌ OUTSTANDING

| Item | Description |
|------|-------------|
| **Features agent** | New agent to implement features from backlog |
| **features.md** | Markdown backlog for new features |
| **ACP integration** | Use Agent Client Protocol for inter-agent comms |
| **Loop sequence** | Change to: remediate → features → test |
| **Infinite loop** | Remove MAX_ITERATIONS cap |
| **10-agent tmux** | Update layout for features agent |

---

## Gaps & Testing Needed

1. **Not tested**: Full loop execution with MiniMax
2. **Not tested**: Failback trigger when MiniMax fails
3. **Not tested**: Linear webhook flow end-to-end
4. **Not tested**: Multi-model critic/validator
5. **Missing**: Features agent + backlog
6. **Missing**: ACP transport setup

---

## Files Summary

```
preprocessing-e2e-test/
├── loop.sh                    # ✅ Ralph loop (claudesp-minimax + failback)
├── ralph-coordination.json    # ✅ State: agents, milestones, failback
├── agents.json                # ✅ 9 agents config
├── swarm-coordinator.md       # ✅ Main coordinator prompt
├── agents/                    # ✅ 9 agent prompts
│   ├── oauth.md
│   ├── environment.md
│   ├── intake-mcp.md
│   ├── tools-validation.md
│   ├── linear-sync.md
│   ├── linear-update.md
│   ├── parity.md
│   ├── critic-observer.md
│   └── failback.md
├── issues/                    # ✅ 9 issue logs
├── test-data/                 # ✅ PRD, arch, research, resources
├── tmux-session.sh            # ✅ 9-pane layout
├── setup-oauth.sh             # ✅ OAuth helper
├── PLAN.md                    # Reference (not modified)
└── README.md                  # ✅ Setup docs
```

---

## Next Actions for MiniMax Agent

1. Create `features.md` backlog with first feature:
   - **Use Agent Client Protocol (ACP)** for multi-agent communication
2. Create `agents/features.md` + `issues/issues-features.md`
3. Update `ralph-coordination.json`: add `features-agent`
4. Update `agents.json`: add features agent entry
5. Update `swarm-coordinator.md`: new sequence (remediate → features → test)
6. Update `tmux-session.sh`: 10-pane layout
7. Update `loop.sh`: remove MAX_ITERATIONS (infinite loop)
