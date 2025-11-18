# Branch Protection Feature

## Overview

The platform automatically protects the `main` branch when creating new projects, ensuring all changes go through pull requests and preventing direct pushes.

## Implementation

### Location

Branch protection is applied in **two places** to ensure coverage:

1. **Intake Workflow** (Primary): `infra/charts/controller/agent-templates/intake/intake.sh`
   - Applied immediately after repository setup and PR creation
   - Ensures protection from the very beginning

2. **Morgan PM** (Secondary): `infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs`
   - Applied during Play workflow project initialization
   - Acts as a backup/double-check for existing projects

### How It Works

#### During Intake (Primary Protection)

When a new project is created via the Intake workflow, these steps occur:

1. **Repository cloned** and project structure created
2. **TaskMaster initialized** with PRD parsing
3. **Branch created** and pushed (e.g., `intake-project-20251118-043000`)
4. **Pull request created** targeting main
5. **Branch protection enabled** ✨ (NEW)
6. Intake complete

#### During Play Workflows (Backup Protection)

When Morgan initializes a project during the Play workflow, it now performs these steps:

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

### GitHub App Permissions Required

For branch protection to work, the Morgan GitHub App needs:

- **Repository permissions**:
  - Administration: Read & write (for branch protection rules)
  - Contents: Read & write (existing requirement)

If the GitHub App lacks these permissions, Morgan will log a warning but continue with project initialization.

## Benefits

1. **Enforces best practices**: No accidental direct pushes to main
2. **Code review required**: All changes must go through PRs
3. **Audit trail**: All changes tracked through PR history
4. **Consistent workflow**: Matches the project's existing policies

## Testing

### Test with Intake Workflow (Primary)

To verify branch protection is enabled during project creation:

1. Trigger an Intake workflow for a new project
2. Check logs for: `🔒 Enabling branch protection on main branch...`
3. Look for success message: `✅ Branch protection enabled on main branch`
4. Verify on GitHub: Settings → Branches → Branch protection rules

### Test with Play Workflow (Backup)

To verify Morgan also applies protection:

1. Trigger a Play workflow
2. Check Morgan PM logs for: `🔒 Configuring branch protection...`
3. Verify protection is applied (or already exists)

### Direct Test

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

