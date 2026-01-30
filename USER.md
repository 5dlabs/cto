# User Expectations

## What I Expect From CTO Lite Agent

### Task Handling

- **Complete autonomy** - Don't stop to ask questions you can answer yourself
- **Read the plan first** - `docs/cto-lite.md` has most answers
- **Verify your work** - Build, test, lint before claiming done

### Code Quality

- **Rust standards:** `cargo clippy --all-targets -- -D warnings` must pass
- **Type safety:** Full TypeScript coverage, no `any`
- **Testing:** Unit tests for critical paths
- **Documentation:** Public APIs must have doc comments

### Git Discipline

- **One feature per commit** - Atomic, revertible changes
- **Clear messages:** `feat(cto-lite): add setup wizard step 1`
- **Push regularly** - Don't let work accumulate locally

### Error Handling

- **Don't panic** - Diagnose, debug, fix
- **Retry failures** - Compilation errors are common, iterate
- **Escalate intelligently** - Only if truly blocked after multiple attempts

## Reference Documents

Always consult these before asking:

1. **`docs/cto-lite.md`** - The comprehensive implementation plan
2. **`templates/skills/`** - Available skill references
3. **`crates/`** - Existing crate patterns to follow

## Sub-Agent Delegation

Feel free to spawn sub-agents for:
- Deep research (use `explore` or `generalPurpose`)
- Parallel implementation tasks
- Code review before commits

## Session Continuity

- Work continuously without stopping
- Only pause for explicit user interruption
- Consider yourself "done" only when the current phase is complete and verified
