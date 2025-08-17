<<<<<<< HEAD
# GitHub Apps Manual Setup Guide

This guide provides step-by-step instructions for manually creating the required GitHub Apps for Task 1. These apps must be created before running the automated task.

## Overview

Task 1 requires four organization-level GitHub Apps to be pre-created with specific permissions and configurations. The automated task will then configure the Helm charts and ExternalSecrets to use these apps.

## Required GitHub Apps

Follow the naming convention: "Persona (5DLabs)". We will add three new apps and reuse the existing Security app.

New apps to create:
1. **Cleo (5DLabs)** - Code quality and formatting specialist
2. **Tess (5DLabs)** - Quality assurance and testing specialist  
3. **Stitch (5DLabs)** - CI/CD failure remediation specialist

Existing apps to reuse:
- **Cipher (5DLabs)** - Security vulnerability remediation specialist (reuse for Security role)
- Existing coding/docs agents: **Rex (5DLabs)**, **Blaze (5DLabs)**, **Morgan (5DLabs)**

## Step-by-Step Creation Process

### For Each GitHub App:

#### 1. Navigate to GitHub App Creation
- Go to: `https://github.com/organizations/5dlabs/settings/apps/new`
- Ensure you're creating in the **5dlabs organization** (not personal account)

#### 2. Basic Information
- **GitHub App name**: Use exact names listed above (e.g., `Cleo (5DLabs)`, `Tess (5DLabs)`, `Stitch (5DLabs)`)
- **Description**: Use the role descriptions from above
- **Homepage URL**: `https://github.com/5dlabs/cto`

#### 3. Repository Permissions
Configure the following permissions for **each app**:

| Permission | Access Level | Reason |
|------------|-------------|---------|
| **Contents** | Read & Write | Read/modify repository files |
| **Issues** | Read & Write | Create/update issues and comments |
| **Pull requests** | Read & Write | Create/update PRs and reviews |
| **Metadata** | Read | Access repository metadata |
| **Actions** | Read | Monitor workflow runs |

#### 4. Organization Permissions
- **Members**: Read (to identify team members)
- **Administration**: Read (to access org settings if needed)

#### 5. Account Permissions
- Leave all account permissions as "No access"

#### 6. Subscribe to Events
Select the following webhook events:
- **Issues** (opened, edited, closed)
- **Pull request** (opened, edited, closed, synchronize)
- **Pull request review** (submitted, edited)
- **Push** (for CI/CD monitoring)
- **Workflow run** (for triage agent)

#### 7. Webhook Configuration
- **Webhook URL**: `https://github.public.5dlabs.ai/github/webhook`
- **Webhook secret**: Use the existing webhook secret from your Argo Events setup

#### 8. Where can this GitHub App be installed?
- Select: **Only on this account (5dlabs)**

#### 9. Create the App
- Click **"Create GitHub App"**
- Note down the **App ID** (you'll need this for the secret store)

#### 10. Generate Private Key
- After creation, scroll down to **"Private keys"** section
- Click **"Generate a private key"**
- Download the `.pem` file (you'll need this for the secret store)

#### 11. Install the App
- Go to **"Install App"** tab
- Click **"Install"** next to the 5dlabs organization
- Select **"All repositories"** or choose specific repositories as needed
- Click **"Install"**

## Required Information to Collect

For each GitHub App, collect the following information:

### Cleo (5DLabs)
- **App ID**: `[RECORD_HERE]`
- **Client ID**: `[RECORD_HERE]` (found in app settings)
- **Private Key**: `[SAVE_PEM_FILE]`
- **Installation ID**: `[RECORD_HERE]` (found after installation)

### Tess (5DLabs)  
- **App ID**: `[RECORD_HERE]`
- **Client ID**: `[RECORD_HERE]`
- **Private Key**: `[SAVE_PEM_FILE]`
- **Installation ID**: `[RECORD_HERE]`

### Stitch (5DLabs)
- **App ID**: `[RECORD_HERE]`
- **Client ID**: `[RECORD_HERE]`
- **Private Key**: `[SAVE_PEM_FILE]`
- **Installation ID**: `[RECORD_HERE]`

### Cipher (5DLabs)
- **App ID**: `[RECORD_HERE]`
- **Client ID**: `[RECORD_HERE]`
- **Private Key**: `[SAVE_PEM_FILE]`
- **Installation ID**: `[RECORD_HERE]`

## Secret Store Configuration

After creating all apps, add their credentials to your external secret store with the following structure:

```yaml
# For each app, create entries like:
github-app-cleo-5dlabs:
  app-id: "[APP_ID]"
  client-id: "[CLIENT_ID]"  
  private-key: |
    -----BEGIN RSA PRIVATE KEY-----
    [PRIVATE_KEY_CONTENT]
    -----END RSA PRIVATE KEY-----
  installation-id: "[INSTALLATION_ID]"

# Repeat for: cleo, tess, stitch (security reuses existing cipher)
```

## Validation Checklist

Before running Task 1, verify:

- [ ] All 4 GitHub Apps are visible at: `https://github.com/organizations/5dlabs/settings/apps`
- [ ] Each app is installed on the 5dlabs organization
- [ ] Each app has the correct repository permissions
- [ ] All App IDs, Client IDs, and Private Keys are recorded
- [ ] Credentials are stored in the external secret store
- [ ] Apps can be accessed via the GitHub API (test with a simple API call)

## Testing App Access

Test each app's access with a simple API call:

```bash
# Generate JWT token (you'll need to implement this)
# Then test installation access:
curl -H "Authorization: Bearer [INSTALLATION_TOKEN]" \
     -H "Accept: application/vnd.github+json" \
     https://api.github.com/installation/repositories
```

## Next Steps

Once all GitHub Apps are created and configured:

1. ✅ Update external secret store with all credentials
2. ✅ Verify apps are installed and have proper permissions  
3. ✅ Run Task 1 - it will now focus on Helm configuration and ExternalSecrets setup
4. ✅ Task 1 will validate app existence and configure the system integration

## Troubleshooting

### Common Issues:
- **Permission Denied**: Ensure you have admin access to the 5dlabs organization
- **App Not Visible**: Check you're creating in the organization, not personal account
- **Installation Failed**: Verify the app has proper permissions before installation
- **API Access Issues**: Ensure private key is correctly formatted and app is installed

### Support:
- GitHub Apps Documentation: https://docs.github.com/en/developers/apps
- GitHub API Reference: https://docs.github.com/en/rest/apps

---

**Note**: This manual setup is required because GitHub Apps cannot be created programmatically via API or CLI. Once created, Task 1 will handle all the automated configuration and integration.
=======
# GitHub Apps Setup Instructions

**Note**: This file documents the GitHub Apps that need to be created. The actual creation requires valid credentials.

## Required GitHub Apps

The following GitHub Apps need to be created with the specified names:

1. **5DLabs-Clippy** (for Cleo - Formatting & Code Quality Specialist)
2. **5DLabs-QA** (for Tess - QA & Testing Specialist) 
3. **5DLabs-Triage** (for Stitch - CI/CD Triage Specialist)
4. **5DLabs-Security** (for Onyx - Security Specialist)

## Setup Process

When valid credentials are available, use one of these methods:

### Method 1: GitHub Web UI
1. Go to https://github.com/organizations/5dlabs/settings/apps
2. Click "New GitHub App"
3. Fill in the required details for each app
4. Generate private keys
5. Store the credentials in the secret store

### Method 2: GitHub CLI (if available)
```bash
# Example for Clippy app
gh api -X POST /orgs/5dlabs/apps \
  -f name="5DLabs-Clippy" \
  -f description="Formatting & Code Quality Specialist" \
  -f homepage_url="https://github.com/5dlabs" \
  -f public=false

# Repeat for other apps...
```

## Required Credentials

Each app needs the following stored in the secret store:
- `app_id`: GitHub App ID
- `private_key`: GitHub App private key (PEM format)
- `client_id`: GitHub App client ID

## Secret Store Keys

The following keys should be created in the secret store:
- `github-app-clippy`
- `github-app-qa` 
- `github-app-triage`
- `github-app-security`

Each with properties: `app_id`, `private_key`, `client_id`
>>>>>>> acf2afa8a2d879a5c44e3843d455261ae174647a
