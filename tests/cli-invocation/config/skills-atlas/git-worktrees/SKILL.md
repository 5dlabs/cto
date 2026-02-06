---
name: git-worktrees
description: Git worktrees create isolated workspaces sharing the same repository for parallel branch work.
---

# Using Git Worktrees

## Overview

Git worktrees create isolated workspaces sharing the same repository, allowing work on multiple branches simultaneously without switching.

**Core principle:** Systematic directory selection + safety verification = reliable isolation.

## When to Use

- Working on multiple features in parallel
- Code review while continuing development
- Testing fixes on different branches
- Isolating experimental work
- Multi-agent development workflows

## Directory Selection Process

### 1. Check Existing Directories

```bash
# Check in priority order
ls -d .worktrees 2>/dev/null    # Preferred (hidden)
ls -d worktrees 2>/dev/null      # Alternative
```

**If found:** Use that directory. If both exist, `.worktrees` wins.

### 2. Check Project Config

Look for worktree directory preference in project documentation (CLAUDE.md, AGENTS.md, etc.).

### 3. Ask User

If no directory exists and no preference specified:

```
No worktree directory found. Where should I create worktrees?
1. .worktrees/ (project-local, hidden)
2. ~/worktrees/<project-name>/ (global location)

Which would you prefer?
```

## Safety Verification

### For Project-Local Directories

**MUST verify directory is ignored before creating worktree:**

```bash
# Check if directory is ignored
git check-ignore -q .worktrees 2>/dev/null || \
git check-ignore -q worktrees 2>/dev/null
```

**If NOT ignored:**
1. Add appropriate line to .gitignore
2. Commit the change
3. Proceed with worktree creation

**Why critical:** Prevents accidentally committing worktree contents to repository.

## Creation Steps

### 1. Detect Project Name

```bash
project=$(basename "$(git rev-parse --show-toplevel)")
```

### 2. Create Worktree

```bash
# Determine full path
case $LOCATION in
  .worktrees|worktrees)
    path="$LOCATION/$BRANCH_NAME"
    ;;
  ~/worktrees/*)
    path="~/worktrees/$project/$BRANCH_NAME"
    ;;
esac

# Create worktree with new branch
git worktree add "$path" -b "$BRANCH_NAME"
cd "$path"
```

### 3. Run Project Setup

Auto-detect and run appropriate setup:

```bash
# Node.js
if [ -f package.json ]; then npm install; fi

# Rust
if [ -f Cargo.toml ]; then cargo build; fi

# Python
if [ -f requirements.txt ]; then pip install -r requirements.txt; fi
if [ -f pyproject.toml ]; then poetry install; fi

# Go
if [ -f go.mod ]; then go mod download; fi
```

### 4. Verify Clean Baseline

Run tests to ensure worktree starts clean:

```bash
# Use project-appropriate command
npm test
cargo test
pytest
go test ./...
```

**If tests fail:** Report failures, ask whether to proceed or investigate.
**If tests pass:** Report ready.

### 5. Report Location

```
Worktree ready at <full-path>
Tests passing (<N> tests, 0 failures)
Ready to implement <feature-name>
```

## Quick Reference

| Situation | Action |
|-----------|--------|
| `.worktrees/` exists | Use it (verify ignored) |
| `worktrees/` exists | Use it (verify ignored) |
| Both exist | Use `.worktrees/` |
| Neither exists | Check config → Ask user |
| Directory not ignored | Add to .gitignore + commit |
| Tests fail during baseline | Report failures + ask |

## Common Commands

```bash
# List all worktrees
git worktree list

# Add worktree with new branch
git worktree add ../feature-branch -b feature/my-feature

# Add worktree with existing branch
git worktree add ../hotfix-branch hotfix/urgent-fix

# Remove worktree (after merging)
git worktree remove ../feature-branch

# Prune stale worktree info
git worktree prune
```

## Common Mistakes

### Skipping ignore verification
- **Problem:** Worktree contents get tracked, pollute git status
- **Fix:** Always use `git check-ignore` before creating project-local worktree

### Assuming directory location
- **Problem:** Creates inconsistency, violates project conventions
- **Fix:** Follow priority: existing > config > ask

### Proceeding with failing tests
- **Problem:** Can't distinguish new bugs from pre-existing issues
- **Fix:** Report failures, get explicit permission to proceed

### Hardcoding setup commands
- **Problem:** Breaks on projects using different tools
- **Fix:** Auto-detect from project files

## Cleanup After Work

When branch is merged:

```bash
# Remove the worktree
git worktree remove .worktrees/feature-branch

# Or if directory was manually deleted
git worktree prune

# Delete the branch if no longer needed
git branch -d feature/my-feature
```

## Red Flags

**Never:**
- Create worktree without verifying it's ignored (project-local)
- Skip baseline test verification
- Proceed with failing tests without asking
- Assume directory location when ambiguous

**Always:**
- Follow directory priority: existing > config > ask
- Verify directory is ignored for project-local
- Auto-detect and run project setup
- Verify clean test baseline

## Attribution

Based on [obra/superpowers](https://github.com/obra/superpowers) using-git-worktrees skill.
