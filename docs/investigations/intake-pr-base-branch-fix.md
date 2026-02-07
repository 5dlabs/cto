# Intake PR Base Branch Detection Fix

## Issue

Morgan intake workflow failed to create PRs for repositories that don't have a `develop` branch:

```
pull request create failed: GraphQL: Head sha can't be blank, Base sha can't be blank,
No commits between develop and intake-prd-alerthub-e2e-test-20260106-113834,
Base ref must be a branch (createPullRequest)
```

## Root Cause

The intake template (`templates/agents/morgan/intake.sh.hbs`) hardcoded the PR base branch to `develop`:

```bash
gh pr create \
    --base "${PR_BASE_BRANCH:-develop}" || echo "⚠️ PR creation failed..."
```

Many repositories (especially new ones) use `main` as their default branch, not `develop`. When the target repo doesn't have a `develop` branch, the GitHub API rejects the PR creation.

## Fix

Updated both templates to detect the repository's default branch with proper fallback order:

### 1. `templates/_shared/partials/git-setup.sh.hbs`

Added smart branch detection that:
1. Uses `PR_BASE_BRANCH` env var if explicitly set
2. Detects the remote's default branch via `git symbolic-ref refs/remotes/origin/HEAD`
3. Falls back to `develop` → `main` → `master` (in that order)

### 2. `templates/agents/morgan/intake.sh.hbs`

Added the same detection logic before PR creation:

```bash
# Detect the repository's default branch for PR base
if [ -z "${PR_BASE_BRANCH:-}" ]; then
  DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
  if [ -n "$DEFAULT_BRANCH" ] && git show-ref --verify --quiet "refs/remotes/origin/$DEFAULT_BRANCH"; then
    PR_BASE_BRANCH="$DEFAULT_BRANCH"
  elif git show-ref --verify --quiet "refs/remotes/origin/develop"; then
    PR_BASE_BRANCH="develop"
  elif git show-ref --verify --quiet "refs/remotes/origin/main"; then
    PR_BASE_BRANCH="main"
  elif git show-ref --verify --quiet "refs/remotes/origin/master"; then
    PR_BASE_BRANCH="master"
  else
    PR_BASE_BRANCH="main"
  fi
  echo "  ℹ️  Detected base branch: $PR_BASE_BRANCH"
fi
```

## Detection Priority

1. **Explicit env var**: If `PR_BASE_BRANCH` is set, use it
2. **Remote HEAD**: `git symbolic-ref refs/remotes/origin/HEAD` gives the actual default branch
3. **develop**: Preferred for repos following gitflow
4. **main**: Modern default for most new repositories
5. **master**: Legacy fallback

## Affected Workflows

- Morgan intake (`intake.sh.hbs`)
- All agent workflows using `git-setup.sh.hbs` partial:
  - Implementation agents (Rex, Blaze, Nova, etc.)
  - Support agents (Cleo, Tess, Atlas, etc.)

## Testing

To verify the fix works:
1. Run intake on a repo with only `main` branch
2. Verify `PR_BASE_BRANCH` is correctly detected as `main`
3. Verify PR is created successfully

## Date

2026-01-06
