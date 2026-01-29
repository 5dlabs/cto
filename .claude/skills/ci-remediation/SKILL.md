---
name: ci-remediation
description: CI remediation and PR health monitoring. Use when watching GitHub CI runs, fixing failing checks, responding to bug-bot comments, addressing Stitch review feedback, or shepherding a PR to merge-ready state.
---

# CI Remediation Skill

Monitor CI runs, fix failing checks, and respond to automated review comments until a PR is ready to merge.

## When to Use

- PR has failing CI checks
- Bug-bot has posted security/quality comments
- Stitch has requested changes
- Need to shepherd a PR through to green CI

## Core Workflow

```
1. Check PR status → gh pr checks
2. Identify failures → Parse check output
3. Fix issues → Edit code, run locally
4. Verify locally → cargo clippy, cargo test, pre-commit
5. Push fixes → git commit && git push
6. Monitor → Loop until all green
```

## Commands

### Check PR Status

```bash
# View all checks with status
gh pr checks

# Detailed status with JSON
gh pr view --json statusCheckRollup,reviews,comments

# Watch checks in real-time
gh pr checks --watch
```

### View Check Logs

```bash
# List workflow runs for PR
gh run list --branch $(git branch --show-current)

# View specific run logs
gh run view <run-id> --log-failed

# Download full logs
gh run view <run-id> --log
```

### Re-run Failed Checks

```bash
# Re-run all failed jobs
gh run rerun <run-id> --failed

# Re-run entire workflow
gh run rerun <run-id>
```

## Common CI Failures

### Clippy (Rust)

```bash
# Run locally with same flags as CI
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Fix auto-fixable issues
cargo clippy --fix --allow-dirty
```

**Common fixes:**
- `unused_imports` → Remove unused imports
- `needless_return` → Use implicit return
- `redundant_clone` → Remove `.clone()`
- `missing_docs` → Add doc comments

### Format Check

```bash
# Check formatting
cargo fmt --all --check

# Auto-fix formatting
cargo fmt --all
```

### Test Failures

```bash
# Run all tests
cargo test

# Run specific test with output
cargo test test_name -- --nocapture

# Run tests for specific crate
cargo test -p crate_name
```

### Pre-commit Hooks

```bash
# Run all hooks
pre-commit run --all-files

# Run specific hook
pre-commit run <hook-id> --all-files

# Common hooks: markdownlint, shellcheck, yaml-lint
```

## Responding to Bug-Bot

Bug-bot posts comments about security and quality issues.

### Identifying Bug-Bot Comments

```bash
# List PR comments
gh pr view --json comments | jq '.comments[] | select(.author.login == "bug-bot")'
```

### Common Bug-Bot Issues

| Issue Type | Resolution |
|------------|------------|
| Hardcoded secret | Move to env var or secrets manager |
| SQL injection risk | Use parameterized queries |
| Missing input validation | Add validation layer |
| Unsafe unwrap | Use proper error handling |

### Response Pattern

1. Read the comment carefully
2. Locate the file and line referenced
3. Apply the fix
4. Reply to the comment (optional): `gh pr comment --body "Fixed in <commit>"`

## Responding to Stitch Reviews

Stitch posts structured code reviews with severity levels.

### Review Severity

- **Critical (Must Fix)** → Security/correctness issues, blocks merge
- **Warning (Should Fix)** → Performance/maintainability issues
- **Suggestion (Consider)** → Optional improvements

### Addressing Reviews

```bash
# View review comments
gh pr view --json reviews | jq '.reviews[] | select(.author.login | contains("Stitch"))'

# View inline comments
gh api repos/{owner}/{repo}/pulls/{pr}/comments | jq '.[] | select(.user.login | contains("Stitch"))'
```

### Resolution Steps

1. Address all **Critical** issues first
2. Fix **Warnings** unless justified to skip
3. Consider **Suggestions** based on scope

After fixes, Stitch re-reviews on the next push.

## Remediation Loop

Run this loop until all checks pass:

```bash
# 1. Check current status
gh pr checks

# 2. If failing, identify the issue
gh run view <run-id> --log-failed

# 3. Fix locally
# ... make edits ...

# 4. Verify locally before pushing
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo test
pre-commit run --all-files

# 5. Commit and push
git add -A && git commit -m "fix: address CI failures" && git push

# 6. Wait for CI and repeat
gh pr checks --watch
```

## Progress Tracking

Track remediation progress:

```markdown
## CI Remediation Progress

PR: #<number>
Branch: <branch-name>

### Checks Status
- [ ] build - ❌ failing
- [ ] clippy - ✅ passing
- [ ] test - ❌ failing
- [ ] fmt - ✅ passing
- [ ] pre-commit - ✅ passing

### Review Comments
- [ ] Bug-bot: SQL injection risk (file.rs:42)
- [ ] Stitch: Missing error handling (api.rs:78)

### Fixes Applied
1. commit-sha - description of fix
```

## Exit Criteria

PR is ready to merge when:

1. All CI checks are green (`gh pr checks` shows all passing)
2. No unresolved Critical/Warning comments from Stitch
3. No unresolved Bug-bot comments
4. PR is in mergeable state (`gh pr view --json mergeable`)

```bash
# Final verification
gh pr view --json statusCheckRollup,mergeable,reviews | jq '{
  allChecksPass: (.statusCheckRollup | all(.conclusion == "SUCCESS")),
  mergeable: .mergeable,
  approved: (.reviews | any(.state == "APPROVED"))
}'
```
