# GitHub App Permissions Requirements

This document outlines the required GitHub App permissions for each agent in the multi-agent orchestration system.

## Overview

Each agent uses a dedicated GitHub App with specific permissions tailored to its responsibilities. These apps must be installed on the target repositories with the correct permissions for the agents to function properly.

## Agent GitHub Apps

### 1. **5DLabs-Rex** (Implementation Agent)
**Purpose**: Create branches, write code, open pull requests

**Required Permissions**:
- **Contents**: Read & Write (to push code)
- **Pull requests**: Read & Write (to create/update PRs)
- **Issues**: Read & Write (to add labels)
- **Metadata**: Read (always required)

### 2. **5DLabs-Cleo** (Code Quality Agent)
**Purpose**: Review code quality, run linters, submit PR reviews

**Required Permissions**:
- **Contents**: Read (to analyze code)
- **Pull requests**: Read & Write (to submit reviews)
- **Checks**: Read (to view CI status)
- **Issues**: Read (to view PR comments)
- **Metadata**: Read (always required)

### 3. **5DLabs-Tess** (QA Testing Agent)
**Purpose**: Run tests, validate acceptance criteria, submit PR approvals

**Required Permissions**:
- **Contents**: Read (to access test files)
- **Pull requests**: Read & Write (to submit reviews and add labels)
- **Issues**: Read & Write (to add labels like 'approved')
- **Deployments**: Read (to verify deployments)
- **Metadata**: Read (always required)

**Critical Note**: If Tess cannot submit PR reviews due to missing permissions, the fallback sensor will trigger on the 'approved' label instead.

### 4. **5DLabs-Cipher** (Security Agent)
**Purpose**: Security review, vulnerability scanning, compliance checks

**Required Permissions**:
- **Contents**: Read (to scan code)
- **Pull requests**: Read & Write (to submit security reviews)
- **Security events**: Read & Write (to report vulnerabilities)
- **Issues**: Read (to view discussions)
- **Metadata**: Read (always required)

### 5. **5DLabs-Atlas** (PR Guardian & Integration)
**Purpose**: Monitor PR lifecycle, resolve conflicts, handle CI failures, auto-merge

**Required Permissions**:
- **Contents**: Read & Write (to resolve conflicts)
- **Pull requests**: Read & Write (to merge PRs and post comments)
- **Issues**: Read & Write (to manage PR comments)
- **Checks**: Read (to monitor CI status)
- **Administration**: Write (for auto-merge if branch protection allows)
- **Metadata**: Read (always required)

**Critical Note**: Atlas requires write permissions to resolve merge conflicts and force-push rebased branches.

## Installation Instructions

### For Organization Repositories

1. Navigate to your GitHub organization settings
2. Go to "Developer settings" → "GitHub Apps"
3. Install each app on the required repositories
4. Grant the permissions listed above for each app

### For Personal Repositories

1. Go to https://github.com/apps/[app-name]
2. Click "Install" or "Configure"
3. Select the repositories where the app should be installed
4. Review and approve the requested permissions

## Troubleshooting

### Common Issues

#### 1. "PR review failed" in Tess logs
- **Cause**: Missing `pull_requests: write` permission
- **Solution**: Update the 5DLabs-Tess app permissions
- **Fallback**: The system will use the 'approved' label trigger instead

#### 2. "Cannot resolve conflicts" in Atlas logs
- **Cause**: Missing `contents: write` permission
- **Solution**: Update the 5DLabs-Atlas app permissions

#### 3. "Cannot create PR" in Rex logs
- **Cause**: Missing `pull_requests: write` or `contents: write` permission
- **Solution**: Update the 5DLabs-Rex app permissions

### Verification Commands

Check if an app is installed and has correct permissions:

```bash
# Check app installation
gh api "/repos/{owner}/{repo}/installation" --jq '.permissions'

# List all installed apps on a repo
gh api "/repos/{owner}/{repo}/installations" --jq '.[] | {id: .id, app: .app_slug, permissions: .permissions}'

# Check specific app permissions (requires admin access)
gh api "/repos/{owner}/{repo}/installation" \
  -H "Accept: application/vnd.github.v3+json" \
  --jq '.permissions | keys'
```

### Diagnostic Script

Use the provided diagnostic script to check for permission issues:

```bash
./infra/scripts/diagnose-tess-atlas-handoff.sh [workflow-name]
```

This will check:
- GitHub App installation status
- Permission availability
- Recent API failures related to permissions

## Security Considerations

- **Principle of Least Privilege**: Each app only has the minimum permissions required
- **Separation of Concerns**: Each agent uses a separate app to isolate permissions
- **Audit Trail**: All actions are logged and attributed to specific apps
- **Token Rotation**: GitHub Apps use short-lived tokens that auto-rotate

## Fallback Mechanisms

The system includes fallback mechanisms for permission failures:

1. **Tess PR Review Fallback**: If Tess cannot submit PR reviews (e.g., due to Cipher requesting changes), the system falls back to label-based triggers via the `tess-label-fallback-sensor`

2. **Atlas Guardian Resumption**: If Atlas cannot be triggered via PR review events, it can be resumed via PR lifecycle events (opened, synchronized, etc.)

3. **Manual Override**: Administrators can manually trigger workflow progression using kubectl commands documented in the runbooks

## Future Enhancements

- [ ] Automated permission verification during agent startup
- [ ] Centralized permission management dashboard
- [ ] Automatic fallback to label-based triggers for all agents
- [ ] Permission escalation workflows for critical operations

---

For additional support, consult the [Atlas Operations Runbook](./runbooks/atlas-operations.md) or contact the platform team.
