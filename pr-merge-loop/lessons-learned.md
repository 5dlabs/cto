# Lessons Learned - PR Merge Loop

This document tracks patterns identified and fixes implemented by the Monitor Agent.

## Format

Each entry follows this structure:

```markdown
### [ISSUE-XXX] Short Description

**Date**: YYYY-MM-DD
**Observation**: What Claude had to do manually
**Root Cause**: Why the code didn't handle it
**Fix Applied**: What was changed
**Files Modified**: List of files
**Status**: fixed|in-progress|blocked
```

---

## Issues

### [ISSUE-001] Rust Linting Not in Pre-commit Hooks

**Date**: 2026-01-22
**Observation**: CI has robust Rust linting (cargo fmt, cargo clippy with pedantic warnings) but pre-commit hooks only covered YAML/GitOps files, markdown, and shell scripts. Rust format/clippy issues could only be caught in CI after PR creation.
**Root Cause**: Pre-commit hooks were initially focused on infrastructure/gitops files when the project started. As the Rust codebase grew, linting wasn't added to pre-commit.
**Fix Applied**: Added `cargo-fmt` and `cargo-clippy` hooks to `.pre-commit-config.yaml` matching the CI `rust-lint` composite action settings.
**Files Modified**: `.pre-commit-config.yaml`
**Status**: fixed

**Rationale**: By catching Rust format and clippy issues before PR creation:
- Bug-bot won't need to comment on clippy warnings
- CI won't fail on format issues
- Developers get faster feedback locally
- PRs are cleaner on first submission
