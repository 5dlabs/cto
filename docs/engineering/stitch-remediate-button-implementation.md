# Stitch Remediate Button Implementation Guide

This document provides implementation details for adding a "Remediate with Rex" button to Stitch's PR review check runs. When clicked, this button triggers Rex to automatically fix the issues identified by Stitch.

## Overview

The flow works as follows:

1. **Stitch reviews PR** → Creates check run with "Remediate with Rex" action button
2. **User clicks button** → GitHub sends `check_run.requested_action` webhook
3. **Rex remediation sensor** → Receives webhook, creates CodeRun CRD
4. **Rex agent** → Fixes issues and pushes commits to PR

---

## Part 1: Creating Check Run with Action Button

### API Call Format

After Stitch completes its review and finds issues, create a check run with an action button:

```bash
# Using gh CLI
gh api "repos/$REPO_SLUG/check-runs" \
    -X POST \
    -H "Accept: application/vnd.github+json" \
    -f name="Stitch Review" \
    -f head_sha="$HEAD_SHA" \
    -f status="completed" \
    -f conclusion="action_required" \
    -f output[title]="$FINDING_COUNT issue(s) found" \
    -f output[summary]="Click **Remediate with Rex** to automatically fix these issues." \
    -f 'actions[][label]=Remediate with Rex' \
    -f 'actions[][description]=Let Rex fix these issues automatically' \
    -f 'actions[][identifier]=remediate'
```

### Using curl (Alternative)

```bash
curl -X POST \
  -H "Authorization: Bearer ${GH_TOKEN}" \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/repos/${REPO_SLUG}/check-runs" \
  -d '{
    "name": "Stitch Review",
    "head_sha": "'"${HEAD_SHA}"'",
    "status": "completed",
    "conclusion": "action_required",
    "output": {
      "title": "'"${FINDING_COUNT}"' issue(s) found",
      "summary": "Click **Remediate with Rex** to automatically fix these issues.\n\n## Issues Found\n\n'"${ISSUES_MARKDOWN}"'"
    },
    "actions": [
      {
        "label": "Remediate with Rex",
        "description": "Let Rex fix these issues automatically",
        "identifier": "remediate"
      }
    ]
  }'
```

### Critical Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `status` | `completed` | **Required** - Button only appears on completed check runs |
| `conclusion` | `action_required` | Shows orange status, indicates user action needed |
| `actions[].identifier` | `remediate` | Must match what the sensor listens for |
| `actions[].label` | `Remediate with Rex` | Button text shown to user |

### Alternative Conclusions

- `action_required` - Orange status, shows button prominently (recommended)
- `failure` - Red status, also shows button
- `success` - Green status, button still works but less visible

---

## Part 2: Conditional Button Creation

Only create the button when there are fixable findings:

```bash
# After Stitch review completes
if [ -f /tmp/review.json ]; then
    FINDING_COUNT=$(jq '.findings | length' /tmp/review.json 2>/dev/null || echo "0")
    
    if [ "$FINDING_COUNT" -gt 0 ]; then
        echo "🔘 Creating check run with Remediate button..."
        
        # Build issues markdown for summary
        ISSUES_MARKDOWN=$(jq -r '.findings[] | "- **\(.severity)**: \(.title) in `\(.file)`"' /tmp/review.json | head -10)
        
        gh api "repos/$REPO_SLUG/check-runs" \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -f name="Stitch Review" \
            -f head_sha="$HEAD_SHA" \
            -f status="completed" \
            -f conclusion="action_required" \
            -f output[title]="$FINDING_COUNT issue(s) found" \
            -f output[summary]="Click **Remediate with Rex** to automatically fix these issues.

## Issues Found

$ISSUES_MARKDOWN" \
            -f 'actions[][label]=Remediate with Rex' \
            -f 'actions[][description]=Let Rex fix these issues automatically' \
            -f 'actions[][identifier]=remediate'
        
        echo "✅ Check run created - user can click 'Remediate with Rex' button"
    else
        # No issues - create success check run without button
        gh api "repos/$REPO_SLUG/check-runs" \
            -X POST \
            -H "Accept: application/vnd.github+json" \
            -f name="Stitch Review" \
            -f head_sha="$HEAD_SHA" \
            -f status="completed" \
            -f conclusion="success" \
            -f output[title]="No issues found" \
            -f output[summary]="Great job! Stitch found no issues in this PR."
    fi
fi
```

---

## Part 3: Webhook Payload Structure

When the user clicks the button, GitHub sends a `check_run` webhook with `action: requested_action`.

### Sample Webhook Payload

```json
{
  "action": "requested_action",
  "requested_action": {
    "identifier": "remediate"
  },
  "check_run": {
    "id": 12345678901,
    "name": "Stitch Review",
    "head_sha": "abc123def456...",
    "status": "completed",
    "conclusion": "action_required",
    "output": {
      "title": "3 issue(s) found",
      "summary": "Click **Remediate with Rex** to automatically fix these issues."
    },
    "pull_requests": [
      {
        "number": 1234,
        "head": {
          "sha": "abc123def456...",
          "ref": "feature/my-branch"
        },
        "base": {
          "sha": "xyz789...",
          "ref": "main"
        }
      }
    ],
    "check_suite": {
      "id": 98765432101,
      "head_branch": "feature/my-branch",
      "head_sha": "abc123def456..."
    },
    "app": {
      "id": 1794583,
      "slug": "5dlabs-stitch",
      "name": "5DLabs-Stitch"
    }
  },
  "repository": {
    "id": 123456789,
    "full_name": "5dlabs/cto",
    "html_url": "https://github.com/5dlabs/cto"
  },
  "sender": {
    "login": "jonathonfritz",
    "id": 12345,
    "type": "User"
  },
  "installation": {
    "id": 56789012
  }
}
```

### Key Fields to Extract

| Field Path | Variable | Description |
|------------|----------|-------------|
| `body.check_run.pull_requests[0].number` | `PR_NUMBER` | The PR number |
| `body.check_run.head_sha` | `HEAD_SHA` | Commit SHA to fix |
| `body.check_run.id` | `CHECK_RUN_ID` | For status updates |
| `body.check_run.name` | `CHECK_RUN_NAME` | Check run name |
| `body.check_run.check_suite.head_branch` | `HEAD_BRANCH` | Branch name |
| `body.repository.full_name` | `REPO_SLUG` | e.g., "5dlabs/cto" |
| `body.repository.html_url` | `REPOSITORY_URL` | For cloning |
| `body.sender.login` | `TRIGGER_USER` | Who clicked button |
| `body.requested_action.identifier` | `ACTION_ID` | Should be "remediate" |

### Important Notes

1. **`pull_requests` is an array** - A commit can be on multiple PRs. Use `[0]` for the first one.
2. **`pull_requests` may be empty** - If the check run was created on a commit not associated with a PR.
3. **`head_branch`** is in `check_suite`, not directly on `check_run`.

---

## Part 4: Rex Remediation Sensor Configuration

### Sensor Filter Configuration

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: rex-remediation
  namespace: automation
spec:
  dependencies:
    - name: check-run-action
      eventSourceName: github
      eventName: org
      filters:
        data:
          # Filter for check_run events
          - path: body.X-GitHub-Event
            type: string
            value:
              - check_run
          # Filter for requested_action (button click)
          - path: body.action
            type: string
            value:
              - requested_action
          # Filter for our specific action identifier
          - path: body.requested_action.identifier
            type: string
            value:
              - remediate
              - rex_remediate
              - fix_issues
```

### Parameter Extraction

```yaml
parameters:
  # PR Number
  - src:
      dependencyName: check-run-action
      dataKey: body.check_run.pull_requests[0].number
    dest: spec.env.PR_NUMBER
  
  # Head SHA
  - src:
      dependencyName: check-run-action
      dataKey: body.check_run.head_sha
    dest: spec.env.HEAD_SHA
  
  # Repository URL
  - src:
      dependencyName: check-run-action
      dataKey: body.repository.html_url
    dest: spec.repositoryUrl
  
  # Repository Slug
  - src:
      dependencyName: check-run-action
      dataKey: body.repository.full_name
    dest: spec.env.REPO_SLUG
  
  # Head Branch
  - src:
      dependencyName: check-run-action
      dataKey: body.check_run.check_suite.head_branch
    dest: spec.env.HEAD_BRANCH
  
  # Trigger Type
  - src:
      dependencyName: check-run-action
      dataTemplate: "check_run_action"
    dest: spec.env.TRIGGER_TYPE
  
  # Who clicked the button
  - src:
      dependencyName: check-run-action
      dataKey: body.sender.login
    dest: spec.env.TRIGGER_USER
  
  # Check Run ID (for status updates)
  - src:
      dependencyName: check-run-action
      dataKey: body.check_run.id
    dest: spec.env.CHECK_RUN_ID
  
  # Check Run Name
  - src:
      dependencyName: check-run-action
      dataKey: body.check_run.name
    dest: spec.env.CHECK_RUN_NAME
```

---

## Part 5: Full Integration Example

### Stitch Review Template Update

Add this to the end of Stitch's review script (`container.sh.hbs` or equivalent):

```bash
#!/bin/bash
set -e

# ... existing Stitch review code ...

# After review completes, check for findings
if [ -f /tmp/review.json ]; then
    FINDING_COUNT=$(jq '.findings | length' /tmp/review.json 2>/dev/null || echo "0")
    CRITICAL_COUNT=$(jq '[.findings[] | select(.severity == "critical")] | length' /tmp/review.json 2>/dev/null || echo "0")
    
    if [ "$FINDING_COUNT" -gt 0 ]; then
        echo ""
        echo "🔘 Creating check run with Remediate button..."
        
        # Determine conclusion based on severity
        if [ "$CRITICAL_COUNT" -gt 0 ]; then
            CONCLUSION="failure"
            TITLE="$CRITICAL_COUNT critical issue(s) found"
        else
            CONCLUSION="action_required"
            TITLE="$FINDING_COUNT issue(s) found"
        fi
        
        # Build summary with issue list
        SUMMARY="Click **Remediate with Rex** to automatically fix these issues.

## Issues Found

$(jq -r '.findings[] | "- **\(.severity)**: \(.title) in \`\(.file)\`:\(.start_line)"' /tmp/review.json | head -15)

---
*Review by Stitch • [View Details](#)*"
        
        # Create check run with action button
        CHECK_RESPONSE=$(curl -s -X POST \
          -H "Authorization: Bearer ${GH_TOKEN}" \
          -H "Accept: application/vnd.github+json" \
          -H "X-GitHub-Api-Version: 2022-11-28" \
          "https://api.github.com/repos/${REPO_SLUG}/check-runs" \
          -d @- <<EOF
{
  "name": "Stitch Review",
  "head_sha": "${HEAD_SHA}",
  "status": "completed",
  "conclusion": "${CONCLUSION}",
  "completed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "output": {
    "title": "${TITLE}",
    "summary": $(echo "$SUMMARY" | jq -Rs .)
  },
  "actions": [
    {
      "label": "Remediate with Rex",
      "description": "Let Rex fix these issues automatically",
      "identifier": "remediate"
    }
  ]
}
EOF
)
        
        CHECK_RUN_ID=$(echo "$CHECK_RESPONSE" | jq -r '.id')
        echo "✅ Check run created: $CHECK_RUN_ID"
        echo "   User can click 'Remediate with Rex' button to trigger Rex"
    else
        echo ""
        echo "✅ No issues found - creating success check run"
        
        curl -s -X POST \
          -H "Authorization: Bearer ${GH_TOKEN}" \
          -H "Accept: application/vnd.github+json" \
          -H "X-GitHub-Api-Version: 2022-11-28" \
          "https://api.github.com/repos/${REPO_SLUG}/check-runs" \
          -d '{
            "name": "Stitch Review",
            "head_sha": "'"${HEAD_SHA}"'",
            "status": "completed",
            "conclusion": "success",
            "completed_at": "'"$(date -u +"%Y-%m-%dT%H:%M:%SZ")"'",
            "output": {
              "title": "No issues found",
              "summary": "Great job! Stitch found no issues in this PR."
            }
          }'
    fi
fi
```

---

## Part 6: Testing

### Manual Test: Create Check Run with Button

```bash
# Set variables
export REPO_SLUG="5dlabs/cto"
export HEAD_SHA="abc123..."  # Get from: git rev-parse HEAD
export GH_TOKEN="ghs_..."    # GitHub App token or PAT

# Create check run with button
gh api "repos/$REPO_SLUG/check-runs" \
    -X POST \
    -f name="Stitch Review (Test)" \
    -f head_sha="$HEAD_SHA" \
    -f status="completed" \
    -f conclusion="action_required" \
    -f output[title]="Test: 1 issue found" \
    -f output[summary]="This is a test check run with a remediate button." \
    -f 'actions[][label]=Remediate with Rex' \
    -f 'actions[][description]=Test button' \
    -f 'actions[][identifier]=remediate'
```

### Verify Button Appears

1. Go to the PR's "Checks" tab
2. Find "Stitch Review (Test)" check
3. Click to expand details
4. Look for "Remediate with Rex" button

### Verify Webhook Delivery

1. Go to GitHub App settings → Advanced → Recent Deliveries
2. Look for `check_run` event with `action: requested_action`
3. Verify payload contains expected fields

---

## Part 7: Troubleshooting

### Button Not Appearing

1. **Check `status`** - Must be `completed`
2. **Check `conclusion`** - Use `action_required` or `failure`
3. **Check `actions` array** - Must have `label`, `description`, and `identifier`

### Webhook Not Received

1. **Verify GitHub App permissions** - Needs "Checks" write permission
2. **Verify webhook subscription** - App must subscribe to `check_run` events
3. **Check webhook URL** - Must be accessible from GitHub

### Sensor Not Triggering

1. **Check filter paths** - `body.X-GitHub-Event` must match header
2. **Check identifier** - Must match exactly what's in the check run
3. **Check sensor logs** - `kubectl logs -n automation -l sensor-name=rex-remediation`

---

## References

- [GitHub Check Runs API](https://docs.github.com/en/rest/checks/runs)
- [GitHub Webhook Events - check_run](https://docs.github.com/en/webhooks/webhook-events-and-payloads#check_run)
- [Argo Events Sensor Documentation](https://argoproj.github.io/argo-events/sensors/filters/)

---

## File Locations

| File | Purpose |
|------|---------|
| `infra/gitops/resources/sensors/rex-remediation-sensor.yaml` | Rex remediation sensor |
| `infra/charts/controller/agent-templates/review/factory/agents.md.hbs` | Stitch agent prompt |
| `infra/charts/controller/agent-templates/review/factory/container.sh.hbs` | Stitch container script |
| `infra/charts/controller/agent-templates/remediate/factory/agents.md.hbs` | Rex remediation prompt |
