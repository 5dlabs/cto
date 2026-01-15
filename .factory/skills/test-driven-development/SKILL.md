---
name: test-driven-development
description: Test-driven development discipline - RED-GREEN-REFACTOR cycle with strict enforcement. Use when implementing any feature or bugfix, before writing implementation code.
---

# Test-Driven Development (TDD)

Write the test first. Watch it fail. Write minimal code to pass.

## When to Use

**Always:**

- New features
- Bug fixes
- Refactoring
- Behavior changes

**Exceptions (ask first):**

- Throwaway prototypes
- Generated code
- Configuration files

Thinking "skip TDD just this once"? Stop. That's rationalization.

---

## The Iron Law

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

Write code before the test? Delete it. Start over.

**No exceptions:**

- Don't keep it as "reference"
- Don't "adapt" it while writing tests
- Don't look at it
- Delete means delete

Implement fresh from tests. Period.

---

## Red-Green-Refactor Cycle

```
┌─────────────────────────────────────────────────────────────┐
│                                                              │
│   RED ──────────► GREEN ──────────► REFACTOR                │
│    │                │                   │                    │
│    │                │                   │                    │
│    ▼                ▼                   ▼                    │
│  Write           Minimal            Clean up                │
│  failing         code to            (tests stay             │
│  test            pass               green)                  │
│    │                │                   │                    │
│    └────────────────┴───────────────────┘                   │
│                     │                                        │
│                     ▼                                        │
│                   NEXT                                       │
│                   TEST                                       │
└─────────────────────────────────────────────────────────────┘
```

### RED - Write Failing Test

Write one minimal test showing what should happen.

**Good:**

```rust
#[test]
fn retries_failed_operations_3_times() {
    let attempts = AtomicUsize::new(0);
    let operation = || {
        let count = attempts.fetch_add(1, Ordering::SeqCst);
        if count < 2 {
            Err(anyhow!("fail"))
        } else {
            Ok("success")
        }
    };

    let result = retry_operation(operation, 3);

    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}
```

Clear name, tests real behavior, one thing.

**Bad:**

```rust
#[test]
fn retry_works() {
    let mock = MockFn::new()
        .returning_err(anyhow!("fail"))
        .returning_err(anyhow!("fail"))
        .returning_ok("success");
    retry_operation(mock, 3);
    assert_eq!(mock.call_count(), 3);
}
```

Vague name, tests mock not code.

**Requirements:**

- One behavior
- Clear name
- Real code (no mocks unless unavoidable)

### Verify RED - Watch It Fail

**MANDATORY. Never skip.**

```bash
cargo test retry_operation -- --nocapture
```

Confirm:

- Test fails (not errors)
- Failure message is expected
- Fails because feature missing (not typos)

**Test passes?** You're testing existing behavior. Fix test.

**Test errors?** Fix error, re-run until it fails correctly.

### GREEN - Minimal Code

Write simplest code to pass the test.

**Good:**

```rust
fn retry_operation<T, E, F>(mut f: F, max_attempts: usize) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for i in 0..max_attempts {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) if i == max_attempts - 1 => return Err(e),
            Err(_) => continue,
        }
    }
    unreachable!()
}
```

Just enough to pass.

**Bad:**

```rust
fn retry_operation<T, E, F>(
    f: F,
    config: RetryConfig,  // YAGNI
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    // Exponential backoff, jitter, logging, metrics...
}
```

Over-engineered.

Don't add features, refactor other code, or "improve" beyond the test.

### Verify GREEN - Watch It Pass

**MANDATORY.**

```bash
cargo test retry_operation
```

Confirm:

- Test passes
- Other tests still pass
- Output pristine (no errors, warnings)

**Test fails?** Fix code, not test.

**Other tests fail?** Fix now.

### REFACTOR - Clean Up

After green only:

- Remove duplication
- Improve names
- Extract helpers

Keep tests green. Don't add behavior.

### Repeat

Next failing test for next feature.

---

## Good Tests

| Quality | Good | Bad |
|---------|------|-----|
| **Minimal** | One thing. "and" in name? Split it. | `test_validates_email_and_domain_and_whitespace` |
| **Clear** | Name describes behavior | `test1` |
| **Shows intent** | Demonstrates desired API | Obscures what code should do |

---

## Why Order Matters

**"I'll write tests after to verify it works"**

Tests written after code pass immediately. Passing immediately proves nothing:

- Might test wrong thing
- Might test implementation, not behavior
- Might miss edge cases you forgot
- You never saw it catch the bug

Test-first forces you to see the test fail, proving it actually tests something.

**"Deleting X hours of work is wasteful"**

Sunk cost fallacy. The time is already gone. Your choice now:

- Delete and rewrite with TDD (X more hours, high confidence)
- Keep it and add tests after (30 min, low confidence, likely bugs)

The "waste" is keeping code you can't trust.

---

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks. Test takes 30 seconds. |
| "I'll test after" | Tests passing immediately prove nothing. |
| "Tests after achieve same goals" | Tests-after = "what does this do?" Tests-first = "what should this do?" |
| "Already manually tested" | Ad-hoc ≠ systematic. No record, can't re-run. |
| "Deleting X hours is wasteful" | Sunk cost fallacy. Keeping unverified code is technical debt. |
| "Keep as reference, write tests first" | You'll adapt it. That's testing after. Delete means delete. |
| "Need to explore first" | Fine. Throw away exploration, start with TDD. |
| "Test hard = design unclear" | Listen to test. Hard to test = hard to use. |
| "TDD will slow me down" | TDD faster than debugging. |
| "Existing code has no tests" | You're improving it. Add tests for existing code. |

---

## Red Flags - STOP and Start Over

- Code before test
- Test after implementation
- Test passes immediately
- Can't explain why test failed
- Tests added "later"
- Rationalizing "just this once"
- "I already manually tested it"
- "Tests after achieve the same purpose"
- "Keep as reference" or "adapt existing code"
- "Already spent X hours, deleting is wasteful"
- "TDD is dogmatic, I'm being pragmatic"
- "This is different because..."

**All of these mean: Delete code. Start over with TDD.**

---

## Example: Bug Fix

**Bug:** Empty email accepted

**RED**

```rust
#[test]
fn rejects_empty_email() {
    let result = validate_email("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Email required");
}
```

**Verify RED**

```bash
$ cargo test rejects_empty_email
FAIL: called `Result::unwrap_err()` on an `Ok` value
```

**GREEN**

```rust
fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.trim().is_empty() {
        return Err(ValidationError::new("Email required"));
    }
    // ...
    Ok(())
}
```

**Verify GREEN**

```bash
$ cargo test rejects_empty_email
PASS
```

**REFACTOR**

Extract validation for multiple fields if needed.

---

## Verification Checklist

Before marking work complete:

- [ ] Every new function/method has a test
- [ ] Watched each test fail before implementing
- [ ] Each test failed for expected reason (feature missing, not typo)
- [ ] Wrote minimal code to pass each test
- [ ] All tests pass
- [ ] Output pristine (no errors, warnings)
- [ ] Tests use real code (mocks only if unavoidable)
- [ ] Edge cases and errors covered

Can't check all boxes? You skipped TDD. Start over.

---

## When Stuck

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write wished-for API. Write assertion first. Ask for help. |
| Test too complicated | Design too complicated. Simplify interface. |
| Must mock everything | Code too coupled. Use dependency injection. |
| Test setup huge | Extract helpers. Still complex? Simplify design. |

---

## Debugging Integration

Bug found? Write failing test reproducing it. Follow TDD cycle. Test proves fix and prevents regression.

Never fix bugs without a test.

---

## Final Rule

```
Production code → test exists and failed first
Otherwise → not TDD
```

No exceptions.
