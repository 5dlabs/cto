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