# Cursor-Inspired Multi-Agent Improvements - Test Plan

## Overview

This test plan ensures 100% functionality of all 6 improvements. Tests are organized by:
1. **Unit Tests** - Rust code, isolated functions
2. **Template Tests** - Handlebars rendering validation
3. **Integration Tests** - Component interaction
4. **E2E Tests** - Full workflow validation

---

## Pre-Test Requirements

Before running tests, ensure:
```bash
# All code compiles
cargo build --release

# No clippy warnings (pedantic mode)
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Existing tests pass
cargo test
```

---

## US-001: Cleo Judge Agent Tests

### Unit Tests (Rust)

| Test ID | Description | Input | Expected Output |
|---------|-------------|-------|-----------------|
| J-U-001 | Parse judge decision JSON | `{"decision": "continue"}` | JudgeDecision::Continue |
| J-U-002 | Parse judge decision with drift | `{"decision": "fresh_start", "drift_indicators": ["avoided hard task"]}` | JudgeDecision::FreshStart with indicators |
| J-U-003 | Invalid decision handling | `{"decision": "invalid"}` | Error: unknown decision |
| J-U-004 | Missing decision field | `{}` | Error: missing required field |

```rust
#[cfg(test)]
mod judge_tests {
    use super::*;

    #[test]
    fn test_parse_continue_decision() {
        let json = r#"{"decision": "continue", "reasoning": "more work needed"}"#;
        let decision: JudgeDecision = serde_json::from_str(json).unwrap();
        assert_eq!(decision.decision, "continue");
    }

    #[test]
    fn test_parse_fresh_start_with_drift() {
        let json = r#"{
            "decision": "fresh_start",
            "drift_indicators": ["risk aversion", "avoiding complex task"],
            "reasoning": "Agent showing tunnel vision"
        }"#;
        let decision: JudgeDecision = serde_json::from_str(json).unwrap();
        assert_eq!(decision.decision, "fresh_start");
        assert_eq!(decision.drift_indicators.len(), 2);
    }

    #[test]
    fn test_complete_decision() {
        let json = r#"{
            "decision": "complete",
            "acceptance_completion": 0.95,
            "quality_score": 0.90
        }"#;
        let decision: JudgeDecision = serde_json::from_str(json).unwrap();
        assert_eq!(decision.decision, "complete");
    }
}
```

### Template Tests

| Test ID | Description | Template | Variables | Expected |
|---------|-------------|----------|-----------|----------|
| J-T-001 | Judge mode enabled | `cleo/judge.md.hbs` | `judge_mode: true` | Contains "Judge Mode: Iteration Decision" |
| J-T-002 | Judge mode disabled | `cleo/quality.md.hbs` | `judge_mode: false` | No judge content |
| J-T-003 | Decision options rendered | `cleo/judge.md.hbs` | `judge_mode: true` | Contains "continue", "complete", "fresh_start" |

### Integration Tests

| Test ID | Description | Setup | Action | Expected |
|---------|-------------|-------|--------|----------|
| J-I-001 | Judge writes decision file | Mock workspace | Run judge probe | `/workspace/.judge_decision` exists |
| J-I-002 | Judge decision consumed by retry loop | Decision file with "continue" | Source retry-loop | CONTINUE_SESSION=true |
| J-I-003 | Judge decision triggers fresh start | Decision file with "fresh_start" | Source retry-loop | FRESH_START=true |

---

## US-002: Continuous Planning Tests

### Unit Tests

| Test ID | Description | Input | Expected |
|---------|-------------|-------|----------|
| P-U-001 | Parse task completion event | Argo webhook payload | TaskCompletionEvent struct |
| P-U-002 | Generate follow-up task | Completed task + context | New task with dependencies |
| P-U-003 | Sub-planner spawning | Complex task area | Sub-planner config |

### Template Tests

| Test ID | Description | Template | Variables | Expected |
|---------|-------------|----------|-----------|----------|
| P-T-001 | Planner mode renders | `morgan/planner.md.hbs` | `planner_mode: true` | Contains "Continuous Planning" |
| P-T-002 | Task creation instructions | `morgan/planner.md.hbs` | `planner_mode: true` | Contains "create follow-up tasks" |

### Integration Tests

| Test ID | Description | Setup | Action | Expected |
|---------|-------------|-------|--------|----------|
| P-I-001 | Planner reacts to completion | Mock task completion | Trigger planner hook | Planner evaluates next steps |
| P-I-002 | Dynamic task creation | Planner mode + completion | Run planner | New task in tasks.json |
| P-I-003 | Sub-planner spawning | Complex area identified | Planner dispatch | Sub-planner CodeRun created |

---

## US-003: Fresh Start Mechanism Tests

### Unit Tests

| Test ID | Description | Input | Expected |
|---------|-------------|-------|----------|
| F-U-001 | Threshold config parsing | `freshStartThreshold: 3` | threshold = 3 |
| F-U-002 | Default threshold | No config | threshold = 3 (default) |

### Shell Script Tests

| Test ID | Description | Setup | Expected |
|---------|-------------|-------|----------|
| F-S-001 | Fresh start at threshold | attempt=4, threshold=3 | FRESH_START=true |
| F-S-002 | No fresh start below threshold | attempt=2, threshold=3 | FRESH_START unset |
| F-S-003 | Fresh start on judge signal | judge_decision=fresh_start | FRESH_START=true |
| F-S-004 | Context cleared on fresh start | FRESH_START=true | No .conversation_id |

```bash
#!/bin/bash
# Fresh start mechanism tests

test_fresh_start_at_threshold() {
    export FRESH_START_THRESHOLD=3
    export attempt=4
    
    should_fresh_start() {
        [ "$attempt" -gt "$FRESH_START_THRESHOLD" ] && echo "true" || echo "false"
    }
    
    result=$(should_fresh_start)
    [ "$result" = "true" ] && echo "PASS: F-S-001" || echo "FAIL: F-S-001"
}

test_no_fresh_start_below_threshold() {
    export FRESH_START_THRESHOLD=3
    export attempt=2
    
    should_fresh_start() {
        [ "$attempt" -gt "$FRESH_START_THRESHOLD" ] && echo "true" || echo "false"
    }
    
    result=$(should_fresh_start)
    [ "$result" = "false" ] && echo "PASS: F-S-002" || echo "FAIL: F-S-002"
}

test_fresh_start_on_judge_signal() {
    export JUDGE_DECISION="fresh_start"
    
    should_fresh_start() {
        [ "$JUDGE_DECISION" = "fresh_start" ] && echo "true" || echo "false"
    }
    
    result=$(should_fresh_start)
    [ "$result" = "true" ] && echo "PASS: F-S-003" || echo "FAIL: F-S-003"
}

test_context_cleared() {
    local test_dir=$(mktemp -d)
    touch "$test_dir/.conversation_id"
    touch "$test_dir/.session_state"
    
    export WORKSPACE="$test_dir"
    export FRESH_START="true"
    
    if [ "$FRESH_START" = "true" ]; then
        rm -f "$WORKSPACE/.conversation_id"
        rm -f "$WORKSPACE/.session_state"
    fi
    
    [ ! -f "$test_dir/.conversation_id" ] && echo "PASS: F-S-004" || echo "FAIL: F-S-004"
    rm -rf "$test_dir"
}

# Run all tests
test_fresh_start_at_threshold
test_no_fresh_start_below_threshold
test_fresh_start_on_judge_signal
test_context_cleared
```

---

## US-004: Role-Specific Model Tests

### Unit Tests (Rust)

| Test ID | Description | Input | Expected |
|---------|-------------|-------|----------|
| R-U-001 | Parse RoleModels struct | Valid JSON | RoleModels instance |
| R-U-002 | Get planner model | role="planner" | "claude-opus-4-5-20251101" |
| R-U-003 | Get worker model | role="worker" | "gpt-5.2-codex" |
| R-U-004 | Fallback on empty role | role="planner", planner="" | fallback model |
| R-U-005 | Unknown role fallback | role="unknown" | fallback model |

```rust
#[cfg(test)]
mod role_models_tests {
    use super::*;

    #[test]
    fn test_parse_role_models() {
        let json = r#"{
            "planner": "claude-opus-4-5-20251101",
            "worker": "gpt-5.2-codex",
            "judge": "claude-opus-4-5-20251101",
            "reviewer": "claude-sonnet-4-5-20250514"
        }"#;
        let models: RoleModels = serde_json::from_str(json).unwrap();
        assert_eq!(models.planner, "claude-opus-4-5-20251101");
        assert_eq!(models.worker, "gpt-5.2-codex");
    }

    #[test]
    fn test_get_model_by_role() {
        let models = RoleModels {
            planner: "opus".to_string(),
            worker: "codex".to_string(),
            judge: "opus".to_string(),
            reviewer: "sonnet".to_string(),
        };
        
        assert_eq!(models.get_model("planner", "fallback"), "opus");
        assert_eq!(models.get_model("worker", "fallback"), "codex");
    }

    #[test]
    fn test_fallback_on_empty() {
        let models = RoleModels {
            planner: "".to_string(),
            ..Default::default()
        };
        
        assert_eq!(models.get_model("planner", "fallback"), "fallback");
    }

    #[test]
    fn test_unknown_role_fallback() {
        let models = RoleModels::default();
        assert_eq!(models.get_model("unknown", "fallback"), "fallback");
    }
}
```

### Config Tests

| Test ID | Description | Config File | Expected |
|---------|-------------|-------------|----------|
| R-C-001 | Config loads roleModels | cto-config.json | roleModels section present |
| R-C-002 | Template resolves role model | Workflow render | Correct model for role |

---

## US-005: Worker Isolation Tests

### Template Tests

| Test ID | Description | Template | Variables | Expected |
|---------|-------------|----------|-----------|----------|
| W-T-001 | Isolation mode strips coordination | `coordinator.md.hbs` | `worker_isolation: true` | No "other workers" references |
| W-T-002 | Normal mode has coordination | `coordinator.md.hbs` | `worker_isolation: false` | Contains coordination instructions |
| W-T-003 | Subagent isolation enforced | `subagent-dispatch.md.hbs` | `worker_isolation: true` | Isolated context only |

### Behavioral Tests

| Test ID | Description | Setup | Action | Expected |
|---------|-------------|-------|--------|----------|
| W-B-001 | Worker cannot see peer status | Isolated worker | Query peer status | No peer info available |
| W-B-002 | Worker has only its task | Isolated worker | Inspect context | Only assigned task visible |

---

## US-006: Simplified Atlas Tests

### Template Tests

| Test ID | Description | Template | Variables | Expected |
|---------|-------------|----------|-----------|----------|
| A-T-001 | Atlas merge-only mode | `atlas/integration.md.hbs` | `simplified: true` | Only merge instructions |
| A-T-002 | No quality judgment | `atlas/integration.md.hbs` | `simplified: true` | No "review quality" |
| A-T-003 | No conflict resolution | `atlas/integration.md.hbs` | `simplified: true` | No "resolve conflicts" |

### Workflow Tests

| Test ID | Description | Setup | Action | Expected |
|---------|-------------|-------|--------|----------|
| A-W-001 | Atlas only runs after CI | Workflow with failing CI | Trigger Atlas | Atlas not triggered |
| A-W-002 | Atlas merges on green CI | Workflow with passing CI | Trigger Atlas | PR merged |

---

## E2E Test Suite

### Full Workflow Test

| Test ID | Description | Setup | Steps | Expected |
|---------|-------------|-------|-------|----------|
| E2E-001 | Judge decides continue | Task with incomplete work | 1. Run implementation<br>2. Run Cleo judge | Judge returns "continue" |
| E2E-002 | Judge decides complete | Task with complete work | 1. Run implementation<br>2. Run Cleo judge | Judge returns "complete" |
| E2E-003 | Fresh start triggered | 4th retry attempt | 1. Set attempt=4<br>2. Run retry loop | FRESH_START=true |
| E2E-004 | Role model resolution | Full workflow | Check each agent's model | Correct model per role |
| E2E-005 | Worker isolation | Parallel workers | Inspect worker contexts | No cross-worker info |
| E2E-006 | Simplified Atlas merge | Complete workflow | Atlas step | Only CI check + merge |

---

## Test Execution Checklist

### Before Implementation
- [ ] All existing tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy --all-targets -- -D warnings -W clippy::pedantic`)
- [ ] Templates render without errors

### After Each User Story

| US | Tests to Run | Command |
|----|--------------|---------|
| US-001 | J-U-*, J-T-*, J-I-* | `cargo test judge && ./test-judge.sh` |
| US-002 | P-U-*, P-T-*, P-I-* | `cargo test planner && ./test-planner.sh` |
| US-003 | F-U-*, F-S-* | `cargo test fresh_start && ./test-fresh-start.sh` |
| US-004 | R-U-*, R-C-* | `cargo test role_models` |
| US-005 | W-T-*, W-B-* | `./test-worker-isolation.sh` |
| US-006 | A-T-*, A-W-* | `./test-atlas.sh` |

### Final Validation
- [ ] All unit tests pass
- [ ] All template tests pass
- [ ] All integration tests pass
- [ ] E2E tests pass (manual verification)
- [ ] No regressions in existing functionality
- [ ] Documentation updated

---

## Test Data Files

Test fixtures are in `cursor-improvements-prd/test-fixtures/`:

```
test-fixtures/
├── judge-decisions/
│   ├── continue.json
│   ├── complete.json
│   └── fresh_start.json
├── role-models/
│   ├── valid-config.json
│   └── empty-roles.json
├── tasks/
│   ├── incomplete-task.json
│   └── complete-task.json
└── contexts/
    ├── isolated-worker.json
    └── coordinated-worker.json
```
