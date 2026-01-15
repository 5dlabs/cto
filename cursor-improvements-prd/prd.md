# Cursor-Inspired Multi-Agent Improvements

## Project Overview

**Branch**: `feat/cursor-multi-agent-improvements`

Implements 6 key improvements from Cursor's research on scaling long-running autonomous coding agents. These improvements address coordination bottlenecks, agent drift, and enable better scaling of parallel agent execution.

## Background

Based on Cursor's January 2026 research article "Scaling long-running autonomous coding", which described running hundreds of concurrent agents for weeks on single projects. Key findings:

- Flat coordination with shared files/locks failed - agents became risk-averse
- Planners + Workers separation worked - continuous planning, focused workers
- Judge agent at end of each cycle is critical for iteration decisions
- Periodic fresh starts combat drift and tunnel vision
- Different models excel at different roles (planning vs coding vs evaluation)
- Removing complexity (integrator role) often helped more than adding it

## User Stories

### US-001: Extend Cleo as Judge Agent (Priority: 0)

**High impact** - Cleo becomes the iteration arbiter.

Extend Cleo's quality review to include iteration judgment with three possible decisions:
- **continue**: More work needed, workers should iterate
- **complete**: Acceptance criteria met, proceed to merge  
- **fresh_start**: Drift detected, restart with clean context

**Acceptance Criteria**:
- [ ] Add judge_mode flag to Cleo's template configuration
- [ ] Create new judge.md.hbs partial in templates/agents/cleo/
- [ ] Implement three decision outputs: continue, complete, fresh_start
- [ ] Add drift detection logic that identifies risk-averse patterns
- [ ] Integrate judge evaluation with acceptance-probe.sh.hbs
- [ ] Add ENABLE_JUDGE_AGENT environment variable
- [ ] Write status to /workspace/.judge_decision file

---

### US-002: Implement Continuous Planning Layer (Priority: 1)

**High impact, high effort** - Morgan shifts from one-shot intake to continuous planning.

**Acceptance Criteria**:
- [ ] Add planner mode to Morgan (templates/agents/morgan/planner.md.hbs)
- [ ] Planner watches for task completion events via Argo workflow hooks
- [ ] Planner can create follow-up tasks dynamically in tasks.json
- [ ] Support spawning sub-planners for complex areas
- [ ] Planner wakes up when tasks complete to evaluate next steps
- [ ] Add continuous_planning flag to cto-config.json
- [ ] Document planner pattern in multi-agent-patterns/SKILL.md

---

### US-003: Fresh Start Mechanism for Drift Recovery (Priority: 2)

**Medium impact, low effort** - Prevents tunnel vision by resetting context.

**Acceptance Criteria**:
- [ ] Add FRESH_START_THRESHOLD config (default: 3 retries)
- [ ] Modify retry-loop.sh.hbs to detect when fresh start is needed
- [ ] When fresh start triggered: CONTINUE_SESSION=false, FRESH_START=true
- [ ] Fresh start clears accumulated context
- [ ] Add drift_detected signal from Cleo judge evaluation
- [ ] Log fresh start events for observability
- [ ] Add freshStartThreshold to cto-config.template.json

---

### US-004: Role-Specific Model Configuration (Priority: 3)

**Medium impact, low effort** - Optimize model selection by role.

Different models excel at different roles:
- Planner: Opus (extended thinking)
- Worker: Codex (fast, focused execution)
- Judge: Opus (reasoning for evaluation)
- Reviewer: Sonnet (balance speed/quality)

**Acceptance Criteria**:
- [ ] Add roleModels section to cto-config.template.json
- [ ] Define model mappings for: planner, worker, judge, reviewer roles
- [ ] Update crates/config/src/types.rs with RoleModels struct
- [ ] Workflow template resolves model by role
- [ ] Add role_model helper to template rendering context

---

### US-005: Enhanced Worker Context Isolation (Priority: 4)

**Medium impact, low effort** - Workers focus only on their task.

Cursor's finding: Workers don't coordinate with other workers or worry about the big picture. They just grind on their assigned task.

**Acceptance Criteria**:
- [ ] Modify coordinator.md.hbs to remove cross-worker awareness
- [ ] Workers receive ONLY their specific task, no coordination info
- [ ] Workers have no information about other workers
- [ ] Update subagent templates to enforce isolation
- [ ] Add worker_isolation flag to subagent dispatch
- [ ] Document isolation principle in multi-agent-patterns/SKILL.md

---

### US-006: Simplify Atlas to Merge-Only Role (Priority: 5)

**Low-medium impact, low effort** - Reduce Atlas bottleneck.

Cursor removed their integrator role - workers handled conflicts themselves. Atlas becomes a simple merge gate.

**Acceptance Criteria**:
- [ ] Reduce Atlas responsibilities to: verify CI, merge PR
- [ ] Remove quality judgment from Atlas (handled by Cleo judge)
- [ ] Remove conflict resolution from Atlas (workers handle themselves)
- [ ] Update templates/agents/atlas/ to reflect simplified role
- [ ] Update play-workflow-template.yaml to skip redundant checks
- [ ] Verify workflow still gates on CI status before merge

---

## Success Gate

All improvements complete when:
- [ ] All 6 user stories pass acceptance criteria
- [ ] Cleo judge mode makes continue/complete/fresh_start decisions
- [ ] Morgan planner mode creates follow-up tasks dynamically
- [ ] Fresh start mechanism triggers after threshold retries
- [ ] Role-specific models are resolved correctly in workflows
- [ ] Workers operate in isolated context without peer awareness
- [ ] Atlas only performs merge verification
- [ ] All changes pass `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
- [ ] All changes pass `cargo test`
- [ ] Documentation updated

## Skills Required

- `cto-platform` - Core platform knowledge
- `rust` - For crate modifications
- `templates` - For Handlebars templates
- `kubernetes` - For Argo workflow changes

## Key Files to Modify

| Component | Files |
|-----------|-------|
| Cleo Judge | `templates/agents/cleo/*.hbs`, `templates/_shared/partials/acceptance-probe.sh.hbs` |
| Continuous Planning | `templates/agents/morgan/planner.md.hbs`, workflow templates |
| Fresh Start | `templates/_shared/partials/retry-loop.sh.hbs` |
| Role Models | `cto-config.template.json`, `crates/config/src/types.rs` |
| Worker Isolation | `templates/_shared/partials/coordinator.md.hbs`, subagent templates |
| Simplified Atlas | `templates/agents/atlas/integration.md.hbs`, workflow templates |

## References

- [Cursor Research Article](https://cursor.com/blog/scaling-autonomous-coding)
- [CTO Multi-Agent Patterns](templates/skills/context/multi-agent-patterns/SKILL.md)
- [CTO Evaluation Skill](templates/skills/context/evaluation/SKILL.md)
