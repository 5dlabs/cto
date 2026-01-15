---
name: verification-before-completion
description: Verification discipline - evidence-based completion claims, running verification commands before asserting success. Use before claiming any work is complete, fixed, or passing.
---

# Verification Before Completion

Claiming work is complete without verification is dishonesty, not efficiency.

## When to Use

**ALWAYS before:**

- ANY variation of success/completion claims
- ANY expression of satisfaction about work state
- Committing, PR creation, task completion
- Moving to next task
- Delegating to other agents

---

## The Iron Law

```
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

If you haven't run the verification command in this response, you cannot claim it passes.

---

## The Gate Function

```
BEFORE claiming any status or expressing satisfaction:

1. IDENTIFY: What command proves this claim?
2. RUN: Execute the FULL command (fresh, complete)
3. READ: Full output, check exit code, count failures
4. VERIFY: Does output confirm the claim?
   - If NO: State actual status with evidence
   - If YES: State claim WITH evidence
5. ONLY THEN: Make the claim

Skip any step = lying, not verifying
```

---

## Common Verification Requirements

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| Tests pass | Test command output: 0 failures | Previous run, "should pass" |
| Linter clean | Linter output: 0 errors | Partial check, extrapolation |
| Build succeeds | Build command: exit 0 | Linter passing, logs look good |
| Bug fixed | Test original symptom: passes | Code changed, assumed fixed |
| Regression test works | Red-green cycle verified | Test passes once |
| Agent completed | VCS diff shows changes | Agent reports "success" |
| Requirements met | Line-by-line checklist | Tests passing |
| Clippy passes | `cargo clippy` with 0 warnings | `cargo build` succeeds |
| Format correct | `cargo fmt --check` exit 0 | Code looks right |

---

## Red Flags - STOP

- Using "should", "probably", "seems to"
- Expressing satisfaction before verification ("Great!", "Perfect!", "Done!")
- About to commit/push/PR without verification
- Trusting agent success reports without checking
- Relying on partial verification
- Thinking "just this once"
- **ANY wording implying success without having run verification**

---

## Rationalization Prevention

| Excuse | Reality |
|--------|---------|
| "Should work now" | RUN the verification |
| "I'm confident" | Confidence ≠ evidence |
| "Just this once" | No exceptions |
| "Linter passed" | Linter ≠ compiler ≠ tests |
| "Agent said success" | Verify independently |
| "Partial check is enough" | Partial proves nothing |
| "Different words so rule doesn't apply" | Spirit over letter |

---

## Key Patterns

### Tests

```
✅ [Run test command] [See: 34/34 pass] "All tests pass"
❌ "Should pass now" / "Looks correct"
```

### Build (Rust)

```
✅ cargo build --release && echo "Build passes: exit 0"
❌ "Linter passed" (linter doesn't check compilation)
```

### Clippy Pedantic

```
✅ cargo clippy --all-targets -- -D warnings -W clippy::pedantic
   [See: 0 warnings] "Clippy pedantic clean"
❌ "Fixed the warning" (without re-running)
```

### Requirements

```
✅ Re-read plan → Create checklist → Verify each → Report gaps or completion
❌ "Tests pass, phase complete"
```

### Agent Delegation

```
✅ Agent reports success → Check VCS diff → Verify changes → Report actual state
❌ Trust agent report
```

---

## CTO Platform Verification Checklist

Before marking any task complete:

### Code Quality

- [ ] `cargo fmt --all --check` - exit 0
- [ ] `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` - 0 warnings
- [ ] `cargo test` - all pass

### Pre-Commit

- [ ] `pre-commit run --all-files` - all hooks pass

### GitOps (if applicable)

- [ ] `make -C infra/gitops validate` - passes

---

## Why This Matters

From failure analysis:

- Trust broken when claims don't match reality
- Undefined functions shipped would crash production
- Missing requirements shipped = incomplete features
- Time wasted: false completion → redirect → rework

---

## The Bottom Line

**No shortcuts for verification.**

Run the command. Read the output. THEN claim the result.

This is non-negotiable.
