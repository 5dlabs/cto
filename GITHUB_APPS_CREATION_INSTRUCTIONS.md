# GitHub Apps Creation Instructions

## Overview
This document provides step-by-step instructions for creating the required GitHub Apps for the multi-agent orchestration system.

## Required GitHub Apps

Create the following four GitHub Apps in the 5DLabs GitHub organization:

### 1. 5DLabs-Clippy
- **Name**: 5DLabs-Clippy
- **Description**: Rust formatting and code quality specialist. Ensures zero Clippy warnings and perfect rustfmt compliance.
- **Homepage URL**: https://github.com/5dlabs/cto

**Permissions**:
- Repository permissions:
  - Contents: Write
  - Pull requests: Write  
  - Issues: Write
  - Metadata: Read
  - Checks: Write

**Subscribe to events**:
- Pull request
- Pull request review comment
- Issue comment
- Workflow run
- Check run

### 2. 5DLabs-QA
- **Name**: 5DLabs-QA
- **Description**: Quality assurance and testing specialist. Writes comprehensive tests and validates acceptance criteria.
- **Homepage URL**: https://github.com/5dlabs/cto

**Permissions**:
- Repository permissions:
  - Contents: Write
  - Pull requests: Write
  - Issues: Write
  - Metadata: Read
  - Checks: Write

**Subscribe to events**:
- Pull request
- Pull request review comment
- Issue comment
- Workflow run
- Check run

### 3. 5DLabs-Triage
- **Name**: 5DLabs-Triage
- **Description**: CI/CD triage and remediation specialist. Fixes failing builds with minimal, surgical changes.
- **Homepage URL**: https://github.com/5dlabs/cto

**Permissions**:
- Repository permissions:
  - Contents: Write
  - Pull requests: Write
  - Issues: Write
  - Metadata: Read
  - Checks: Write
  - Actions: Write

**Subscribe to events**:
- Pull request
- Pull request review comment
- Issue comment
- Workflow run
- Check run

### 4. 5DLabs-Security
- **Name**: 5DLabs-Security
- **Description**: Security and vulnerability specialist. Remediates security issues, removes exposed secrets, and applies least-privilege fixes.
- **Homepage URL**: https://github.com/5dlabs/cto

**Permissions**:
- Repository permissions:
  - Contents: Write
  - Pull requests: Write
  - Issues: Write
  - Metadata: Read
  - Checks: Write
  - Security events: Write

**Subscribe to events**:
- Pull request
- Pull request review comment
- Issue comment
- Workflow run
- Check run
- Security advisory

## Creation Steps

### Method 1: Manual Creation (Recommended)

1. Navigate to: https://github.com/organizations/5dlabs/settings/apps
2. Click "New GitHub App"
3. Fill in the details for each app as specified above
4. Set "Where can this GitHub App be installed?" to "Only on this account"
5. Click "Create GitHub App"
6. Generate a private key and save it securely
7. Install the app on the 5dlabs organization with access to appropriate repositories
8. Note down the App ID, Client ID, and save the private key

### Method 2: API Creation (Advanced)

Use the GitHub Apps manifest flow:

```bash
# Create manifest for each app
curl -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_ADMIN_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/app-manifests/{code}/conversions \
  -d @app-manifest.json
```

Note: This requires implementing the complete manifest flow with temporary codes.

## Post-Creation Steps

After creating each GitHub App:

1. **Record the credentials**:
   - App ID
   - Client ID  
   - Private Key (PEM format)

2. **Store credentials in secret store** under these keys:
   - `github-app-clippy` (app_id, client_id, private_key)
   - `github-app-qa` (app_id, client_id, private_key)
   - `github-app-triage` (app_id, client_id, private_key)
   - `github-app-security` (app_id, client_id, private_key)

3. **Install the apps** on the 5DLabs organization with access to relevant repositories

4. **Verify ExternalSecrets sync** by checking:
   ```bash
   kubectl -n agent-platform get secrets | grep github-app-5dlabs
   ```

## Validation

Once all apps are created and credentials stored:

1. **Check app visibility**:
   - Visit: https://github.com/organizations/5dlabs/settings/apps
   - Verify all four apps are listed

2. **Check installations**:
   - Each app should show "Installed" status
   - Should have access to appropriate repositories

3. **Verify ConfigMap update**:
   ```bash
   kubectl -n agent-platform get cm controller-agents -o yaml | grep -E "(Clippy|QA|Triage|Security)"
   ```

## Troubleshooting

- **"App name already taken"**: The app names must be globally unique on GitHub
- **Permission denied**: Ensure you have organization owner permissions
- **Secret sync fails**: Check that the ExternalSecrets operator is running and the secret store is accessible

## Security Notes

- Private keys should be stored securely and never committed to git
- Use the principle of least privilege for app permissions
- Regularly rotate private keys if possible
- Monitor app activity through GitHub's security logs