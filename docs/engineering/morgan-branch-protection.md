# Morgan Branch Protection Feature

## Overview

Morgan now automatically protects the `main` branch when initializing projects, ensuring all changes go through pull requests and preventing direct pushes.

## Implementation

### Location

- **Helper Function**: `infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs`
- **Integration**: `infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs`

### How It Works

When Morgan initializes a new project during the Play workflow, it now performs these steps:

1. **Creates GitHub Project** (existing functionality)
2. **Sets up custom fields** (existing functionality)
3. **Enables branch protection** (NEW)
4. **Stores project configuration** (existing functionality)

### Branch Protection Configuration

The protection rules applied to `main` branch:

- ✅ **Require pull request reviews** (1 approval required)
- ✅ **Dismiss stale reviews** when new commits are pushed
- ✅ **Enforce for admins** - even admin users must use PRs
- ✅ **Block force pushes** - no rewriting history
- ✅ **Block branch deletion** - main branch cannot be deleted
- ⚠️ **Status checks**: None required by default (can be configured later)
- ⚠️ **CODEOWNERS review**: Not required by default (can be configured later)

### Error Handling

The branch protection feature is designed to be resilient:

- **Branch doesn't exist yet**: Logs a warning but continues (protection will need to be enabled after first push)
- **Insufficient permissions**: Logs a warning but continues (doesn't fail the workflow)
- **API errors**: Logged for debugging but won't block project initialization

### Requirements

For branch protection to work, **both** of these are required:

1. **Repository must be public OR organization must have GitHub Pro/Team/Enterprise**
   - GitHub's free plan only allows branch protection on public repositories
   - Private repositories require a paid GitHub plan
   
2. **Morgan GitHub App permissions**:
   - Administration: Read & write (for branch protection rules)
   - Contents: Read & write (existing requirement)

If either requirement is not met, Morgan will log a specific warning but continue with project initialization.

## Benefits

1. **Enforces best practices**: No accidental direct pushes to main
2. **Code review required**: All changes must go through PRs
3. **Audit trail**: All changes tracked through PR history
4. **Consistent workflow**: Matches the project's existing policies

## Testing

To verify branch protection is enabled:

1. Trigger a Morgan workflow
2. Check logs for: `🔒 Enabling branch protection for owner/repo:main`
3. Verify on GitHub: Settings → Branches → Branch protection rules

Or test directly:

```bash
# This should fail with branch protection enabled
git checkout main
git commit --allow-empty -m "test direct push"
git push origin main
# Expected: remote: error: GH006: Protected branch update failed
```

## Configuration

The default branch to protect is `main`, but this can be modified in the helper function:

```bash
enable_branch_protection "$REPO_OWNER" "$REPO_NAME" "main"  # or "master", "develop", etc.
```

## Future Enhancements

Potential improvements:

- Make required approvals count configurable
- Add required status checks from environment variables
- Support protecting multiple branches
- Enable CODEOWNERS review requirement
- Configure allowed merge strategies (squash, rebase, merge commit)

